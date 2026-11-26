import { HttpClient } from '@angular/common/http';
import { Component, Inject, OnDestroy, OnInit } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { ConfigService, PORTMASTER_HTTP_API_ENDPOINT, Setting } from '@safing/portmaster-api';
import { Subscription } from 'rxjs';
import { StatusService, VersionStatus } from 'src/app/services';
import { ActionIndicatorService } from 'src/app/shared/action-indicator';
import { fadeInAnimation } from 'src/app/shared/animations';
import { SaveSettingEvent } from 'src/app/shared/config/generic-setting/generic-setting';
import { I18nService, Language } from 'src/i18n';

export interface UiFlavorInfo {
  id: string;
  name: string;
  kind: string;
  installed: boolean;
  running: boolean;
  active: boolean;
  liveRamMb?: number | null;
  liveRssMb?: number | null;
  benchmarkRamMb: number;
  browserTabRamMb?: number | null;
  processCount: number;
  url?: string;
}

export interface UiFlavorsReport {
  activeFlavor: string;
  keepApiPortOpen: boolean;
  apiPortListening: boolean;
  apiPortAddress: string;
  unixSocketPath?: string;
  flavors: UiFlavorInfo[];
}

@Component({
  templateUrl: './settings.html',
  styleUrls: [
    '../page.scss',
    './settings.scss'
  ],
  animations: [fadeInAnimation]
})
export class SettingsComponent implements OnInit, OnDestroy {
  /** @private The current search term for the settings. */
  searchTerm: string = '';

  /** @private All settings currently displayed. */
  settings: Setting[] = [];

  /** @private The available and selected resource versions. */
  versions: VersionStatus | null = null;

  /**
   * @private
   * The key of the setting to highligh, if any ...
   */
  highlightSettingKey: string | null = null;

  /** @private UI flavor & live RAM telemetry */
  uiReport: UiFlavorsReport | null = null;
  uiBusy = false;

  /** Subscription to watch all available settings. */
  private subscription = Subscription.EMPTY;

  constructor(
    public configService: ConfigService,
    public statusService: StatusService,
    private actionIndicator: ActionIndicatorService,
    private route: ActivatedRoute,
    private i18n: I18nService,
    private http: HttpClient,
    @Inject(PORTMASTER_HTTP_API_ENDPOINT) private httpAPI: string,
  ) { }

  /** @private Currently selected interface language. */
  get language(): Language {
    return this.i18n.language;
  }

  /** @private Switch interface language; persisted by I18nService. */
  setLanguage(language: Language): void {
    this.i18n.setLanguage(language);
  }

  ngOnInit(): void {
    this.subscription = new Subscription();

    this.loadSettings();
    this.refreshUiReport();

    // Request the current resource versions once. We add
    // it to the subscription to prevent a memory leak in
    // case the user leaves the page before the versions
    // have been loaded.
    const versionSub = this.statusService.getVersions()
      .subscribe(version => this.versions = version);

    this.subscription.add(versionSub);

    const querySub = this.route.queryParamMap
      .subscribe(
        params => {
          this.highlightSettingKey = params.get('setting');
        }
      )
    this.subscription.add(querySub);
  }

  ngOnDestroy() {
    this.subscription.unsubscribe();
  }

  /**
   * Loads all settings from the portmaster.
   */
  private loadSettings() {
    const configSub = this.configService.query('')
      .subscribe(settings => this.settings = settings);
    this.subscription.add(configSub);
  }

  /**
   * @private
   * SaveSettingEvent is emitted by the settings-view
   * component when a value has been changed and should be saved.
   *
   * @param event The save-settings event
   */
  saveSetting(event: SaveSettingEvent) {
    let idx = this.settings.findIndex(setting => setting.Key === event.key);
    if (idx < 0) {
      return;
    }

    const setting = {
      ...this.settings[idx],
    }

    if (event.isDefault) {
      delete (setting['Value']);
    } else {
      setting.Value = event.value;
    }

    this.configService.save(setting)
      .subscribe({
        next: () => {
          if (!!event.accepted) {
            event.accepted();
          }

          this.settings[idx] = setting;

          // copy the settings into a new array so we trigger
          // an input update due to changed array identity.
          this.settings = [...this.settings];

          // for the release level setting we need to
          // to a page-reload since portmaster will now
          // return more settings.
          if (setting.Key === 'core/releaseLevel') {
            this.loadSettings();
          }
          if (setting.Key === 'core/uiFlavor' || setting.Key === 'core/keepApiPortOpen') {
            this.refreshUiReport();
          }
        },
        error: err => {
          if (!!event.rejected) {
            event.rejected(err);
          }

          this.actionIndicator.error('Failed to save setting', err);
          console.error(err);
        }
      })
  }

  refreshUiReport(): void {
    const sub = this.http
      .get<UiFlavorsReport>(`${this.httpAPI}/v1/ui/flavors`)
      .subscribe({
        next: report => (this.uiReport = report),
        error: () => { /* older core without /ui/flavors */ },
      });
    this.subscription.add(sub);
  }

  selectUiFlavor(flavorId: string): void {
    this.uiBusy = true;
    const sub = this.http
      .post<UiFlavorsReport>(`${this.httpAPI}/v1/ui/flavor`, { flavor: flavorId })
      .subscribe({
        next: report => {
          this.uiReport = report;
          this.uiBusy = false;
        },
        error: err => {
          this.uiBusy = false;
          this.actionIndicator.error('Failed to switch UI flavor', err);
        },
      });
    this.subscription.add(sub);
  }

  toggleKeepApiPortOpen(): void {
    if (!this.uiReport) return;
    const next = !this.uiReport.keepApiPortOpen;
    this.uiBusy = true;
    const sub = this.http
      .post(`${this.httpAPI}/v1/core/api-port`, { keepOpen: next })
      .subscribe({
        next: () => {
          this.uiBusy = false;
          this.refreshUiReport();
        },
        error: err => {
          this.uiBusy = false;
          this.actionIndicator.error('Failed to update API port setting', err);
        },
      });
    this.subscription.add(sub);
  }

  openWebFlavor(flavor: UiFlavorInfo): void {
    if (flavor.url) {
      window.location.href = flavor.url;
    }
  }
}
