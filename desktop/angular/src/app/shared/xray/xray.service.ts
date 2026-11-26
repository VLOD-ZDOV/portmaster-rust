import { HttpClient } from '@angular/common/http';
import { Inject, Injectable } from '@angular/core';
import { BoolSetting, ConfigService, PORTMASTER_HTTP_API_ENDPOINT, PortapiService } from '@safing/portmaster-api';
import { BehaviorSubject, EMPTY, Observable, catchError, debounceTime, merge, shareReplay, skip, switchMap, tap, timer } from 'rxjs';
import { XrayNode, XrayStatus, XraySubscription } from './xray.types';

/**
 * Frontend contract for the Xray backend. The backend exposes:
 *   GET  xray/subscriptions
 *   POST xray/subscriptions   (add a URL)
 *   GET  xray/nodes
 *   POST xray/connect
 *   GET  xray/status
 *
 * All paths are relative to the Portmaster HTTP API endpoint and use the
 * same /v1 prefix as the other API services in this application.
 */
@Injectable({ providedIn: 'root' })
export class XrayService {
  private readonly basePath: string;
  private readonly apiBase: string;
  private readonly reload$ = new BehaviorSubject<void>(undefined);

  readonly subscriptions$: Observable<XraySubscription[]> = this.reload$.pipe(
    switchMap(() => this.http.get<XraySubscription[]>(`${this.basePath}/subscriptions`)),
  );

  readonly nodes$: Observable<XrayNode[]> = this.reload$.pipe(
    switchMap(() => this.http.get<XrayNode[]>(`${this.basePath}/nodes`)),
  );

  readonly status$: Observable<XrayStatus> = merge(
    this.reload$.pipe(skip(1)),
    timer(0, 5000),
    this.configService.watch<BoolSetting>('spn/enable').pipe(
      skip(1),
      debounceTime(150),
      catchError(() => EMPTY),
    ),
    this.portapi.watch('runtime:spn/status', { ignoreDelete: true }).pipe(
      skip(1),
      catchError(() => EMPTY),
    ),
  ).pipe(
    switchMap(() => this.http.get<XrayStatus>(`${this.basePath}/status`).pipe(
      catchError(() => EMPTY),
    )),
    // Share live Xray state across header, dashboard, and map page, and stay
    // in sync when tray or other tabs toggle spn/enable.
    shareReplay({ bufferSize: 1, refCount: true }),
  );

  constructor(
    private readonly http: HttpClient,
    private readonly configService: ConfigService,
    private readonly portapi: PortapiService,
    @Inject(PORTMASTER_HTTP_API_ENDPOINT) httpAPI: string,
  ) {
    this.apiBase = `${httpAPI}/v1`;
    this.basePath = `${this.apiBase}/xray`;
  }

  /** Add a subscription by URL. */
  addSubscription(url: string): Observable<XraySubscription> {
    return this.http.post<XraySubscription>(`${this.basePath}/subscriptions`, { url })
      .pipe(tap(() => this.refresh()));
  }

  /** Remove a subscription by id. */
  removeSubscription(id: string): Observable<unknown> {
    return this.http.delete(`${this.basePath}/subscriptions/${encodeURIComponent(id)}`)
      .pipe(tap(() => this.refresh()));
  }

  /** Connect to a single node from any subscription. */
  connect(nodeId: string): Observable<unknown> {
    return this.http.post(`${this.basePath}/connect`, { id: nodeId, nodeId })
      .pipe(tap(() => this.refresh()));
  }

  /** Reconnect to the last selected Xray node. */
  connectLast(): Observable<unknown> {
    return this.http.post(`${this.basePath}/connect-last`, {})
      .pipe(tap(() => this.refresh()));
  }

  /** Disconnect active Xray node and stop process. */
  disconnect(): Observable<unknown> {
    return this.http.post(`${this.basePath}/disconnect`, {})
      .pipe(tap(() => this.refresh()));
  }

  /** Download / update the official Xray-core binary into Portmaster bin directory. */
  downloadBinary(): Observable<{ success: boolean; path: string; version?: string }> {
    return this.http.post<{ success: boolean; path: string; version?: string }>(`${this.basePath}/download-binary`, {})
      .pipe(tap(() => this.refresh()));
  }

  /** Reset per-app VPN (spn/use) and Split Tunnel (splittun/use) overrides across all apps. */
  resetRoutingOverrides(): Observable<{ updated: number; profiles: string[] }> {
    return this.http.post<{ updated: number; profiles: string[] }>(`${this.apiBase}/profiles/reset-routing`, {});
  }

  /** Ask the backend to re-fetch all subscription URLs. */
  refresh(): void {
    this.reload$.next();
  }
}
