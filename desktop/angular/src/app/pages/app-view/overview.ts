import {
  ChangeDetectorRef,
  Component,
  OnDestroy,
  OnInit,
  TrackByFunction,
} from '@angular/core';
import {
  AppProfile,
  AppProfileService,
  BoolSetting,
  ConfigService,
  getAppSetting,
  Netquery,
  setAppSetting,
  trackById,
} from '@safing/portmaster-api';
import { SfngDialogService } from '@safing/ui';
import { BehaviorSubject, Subscription, combineLatest, forkJoin, of } from 'rxjs';
import { catchError, debounceTime, filter, startWith, switchMap } from 'rxjs/operators';
import {
  fadeInAnimation,
  fadeInListAnimation,
  moveInOutListAnimation,
} from 'src/app/shared/animations';
import { FuzzySearchService } from 'src/app/shared/fuzzySearch';
import { EditProfileDialog } from './../../shared/edit-profile-dialog/edit-profile-dialog';
import { coerceBooleanProperty } from '@angular/cdk/coercion';
import { MergeProfileDialogComponent } from './merge-profile-dialog/merge-profile-dialog.component';
import { ActionIndicatorService } from 'src/app/shared/action-indicator';
import { I18nService } from 'src/i18n';
import { Router } from '@angular/router';
import {
  ImportConfig,
  ImportDialogComponent,
} from 'src/app/shared/config/import-dialog/import-dialog.component';
import { XrayService } from 'src/app/shared/xray/xray.service';

type AppVpnOverride = 'default' | 'vpn' | 'direct';
type RouteFilter = 'all' | 'vpn' | 'direct' | 'custom' | 'splittun';

interface LocalAppProfile extends AppProfile {
  hasConfigChanges: boolean;
  selected: boolean;
  spnOverride: AppVpnOverride;
  effectiveVpn: boolean;
  splitOverride: boolean | null;
  effectiveSplitTun: boolean;
}

@Component({
  selector: 'app-settings-overview',
  templateUrl: './overview.html',
  styleUrls: ['../page.scss', './overview.scss'],
  animations: [fadeInAnimation, fadeInListAnimation, moveInOutListAnimation],
})
export class AppOverviewComponent implements OnInit, OnDestroy {
  private subscription = Subscription.EMPTY;

  /** Whether or not we are currently loading */
  loading = true;

  /** All application profiles that are actually running */
  runningProfiles: LocalAppProfile[] = [];

  /** All application profiles that have been edited recently */
  recentlyEdited: LocalAppProfile[] = [];

  /** All application profiles */
  profiles: LocalAppProfile[] = [];

  /** The current search term */
  searchTerm = '';

  /** total number of profiles */
  total = 0;

  globalRouteByDefault = false;
  xrayActive = false;
  splitTunEnabled = false;
  globalSplitTunUse = false;
  routingFilter: RouteFilter = 'all';
  routeCounts: Record<RouteFilter, number> = {
    all: 0,
    vpn: 0,
    direct: 0,
    custom: 0,
    splittun: 0,
  };
  resettingRouting = false;

  /** Whether or not we are in profile-selection mode */
  set selectMode(v: any) {
    this._selectMode = coerceBooleanProperty(v);

    // reset all previous profile selections
    if (!this._selectMode) {
      this.profiles.forEach((profile) => (profile.selected = false));
    }
  }
  get selectMode() {
    return this._selectMode;
  }
  private _selectMode = false;

  get selectedProfileCount() {
    return this.profiles.reduce(
      (sum, profile) => (profile.selected ? sum + 1 : sum),
      0
    );
  }

  /** Observable emitting the search term */
  private onSearch = new BehaviorSubject('');
  private onFilterChange = new BehaviorSubject<RouteFilter>('all');

  /** TrackBy function for the profiles. */
  trackProfile: TrackByFunction<LocalAppProfile> = trackById;

  constructor(
    private profileService: AppProfileService,
    private changeDetector: ChangeDetectorRef,
    private searchService: FuzzySearchService,
    private netquery: Netquery,
    private dialog: SfngDialogService,
    private actionIndicator: ActionIndicatorService,
    private configService: ConfigService,
    private xrayService: XrayService,
    private router: Router,
    private i18n: I18nService
  ) { }

  handleProfileClick(profile: LocalAppProfile, event: MouseEvent) {
    if (event.shiftKey) {
      // stay on the same page as clicking the app actually triggers
      // a navigation before this handler is executed.
      this.router.navigate(['/app/overview']);

      this.selectMode = true;

      event.preventDefault();
      event.stopImmediatePropagation();
      event.stopPropagation();
    }

    if (this.selectMode) {
      profile.selected = !profile.selected;
    }

    if (event.shiftKey && this.selectedProfileCount === 0) {
      this.selectMode = false;
    }
  }

  importProfile() {
    const importConfig: ImportConfig = {
      type: 'profile',
      key: '',
    };

    this.dialog.create(ImportDialogComponent, {
      data: importConfig,
      autoclose: false,
      backdrop: 'light',
    });
  }

  stripHtmlTags(text: string): string {
    if (!text) return '';  
    // Only strip if we have proper HTML tags (opening and closing with same tag name)
    return text.replace(/<([a-zA-Z][a-zA-Z0-9]*)[^>]*>([^<]*)<\/\1>/g, '$2');
  }

  openMergeDialog() {
    this.dialog.create(MergeProfileDialogComponent, {
      autoclose: true,
      backdrop: 'light',
      data: this.profiles.filter((p) => p.selected)
      .map((p) => ({
        ...p,
        // Strip HTML tags from Name if it exists (e.g., highlighted search results)
        ...(p.Name ? { Name: this.stripHtmlTags(p.Name) } : {})
      })), 
    });

    this.selectMode = false;
  }

  deleteSelectedProfiles() {
    this.dialog
      .confirm({
        header: this.i18n.t('app.deleteProfilesConfirm'),
        message: this.i18n.t('app.deleteProfilesConfirmMsg', { count: this.selectedProfileCount }),
        caption: this.i18n.t('app.attention'),
        buttons: [
          {
            id: 'no',
            text: this.i18n.t('common.cancel'),
            class: 'outline',
          },
          {
            id: 'yes',
            text: this.i18n.t('common.delete'),
            class: 'danger',
          },
        ],
      })
      .onAction('yes', () => {
        forkJoin(
          this.profiles
            .filter((profile) => profile.selected)
            .map((p) => this.profileService.deleteProfile(p))
        ).subscribe({
          next: () => {
            this.actionIndicator.success(
              this.i18n.t('app.profilesDeleted'),
              this.i18n.t('app.profilesDeletedMsg')
            );
          },
          error: (err) => {
            this.actionIndicator.error(
              this.i18n.t('app.deleteProfilesFailed'),
              this.i18n.t('app.deleteProfilesFailedMsg', {
                error: this.actionIndicator.getErrorMessgae(err),
              })
            );
          },
        });
      })
      .onClose.subscribe(() => (this.selectMode = false));
  }

  ngOnInit() {
    // watch all profiles and re-emit (debounced) when the user
    // enters or chanages the search-text.
    this.subscription = combineLatest([
      this.profileService.watchProfiles().pipe(
        catchError((err) => {
          console.error('[overview] watchProfiles error:', err);
          return of([] as AppProfile[]);
        })
      ),
      this.onSearch.pipe(debounceTime(100), startWith('')),
      this.netquery.getActiveProfileIDs().pipe(
        catchError(() => of([] as string[])),
        startWith([] as string[])
      ),
      this.configService.watch<BoolSetting>('spn/use').pipe(
        catchError(() => of(false)),
        startWith(false)
      ),
      this.configService.watch<BoolSetting>('spn/enable').pipe(
        catchError(() => of(false)),
        startWith(false)
      ),
      this.configService.watch<BoolSetting>('splittun/enable').pipe(
        catchError(() => of(false)),
        startWith(false)
      ),
      this.configService.watch<BoolSetting>('splittun/use').pipe(
        catchError(() => of(false)),
        startWith(false)
      ),
      this.onFilterChange,
    ]).subscribe(([profiles, searchTerm, activeProfiles, globalSpnUse, spnEnable, splitTunEnable, globalSplitTunUse, routingFilter]) => {
      this.loading = false;
      this.globalRouteByDefault = !!globalSpnUse;
      this.xrayActive = !!spnEnable;
      this.splitTunEnabled = !!splitTunEnable;
      this.globalSplitTunUse = !!globalSplitTunUse;
      this.routingFilter = routingFilter;

      const validProfiles = (profiles || [])
        .filter((profile): profile is AppProfile => !!profile && !!profile.ID && (profile.Source as string) !== 'special' && profile.ID !== 'global-config')
        .map((profile) => ({
          ...profile,
          Name: profile.Name || profile.PresentationPath || profile.ID || '',
        }));

      const activeList = Array.isArray(activeProfiles) ? activeProfiles : [];

      // find all profiles that match the search term. For searchTerm="" thsi
      // will return all profiles.
      const filtered = this.searchService.searchList(validProfiles, searchTerm, {
        ignoreLocation: true,
        ignoreFieldNorm: true,
        threshold: 0.1,
        minMatchCharLength: 3,
        keys: ['Name', 'PresentationPath'],
      });

      // create a lookup map of all profiles we already loaded so we don't loose
      // selection state when a profile has been updated.
      const oldProfiles = new Map<string, LocalAppProfile>(
        this.profiles.map((profile) => [
          `${profile.Source}/${profile.ID}`,
          profile,
        ])
      );

      // Prepare new, empty lists for our groups
      this.profiles = [];
      this.runningProfiles = [];
      this.recentlyEdited = [];

      // calcualte the threshold for "recently-used" (1 week).
      const recentlyUsedThreshold =
        new Date().valueOf() / 1000 - 60 * 60 * 24 * 7;

      const sortedLocal: LocalAppProfile[] = filtered
        .map((item) => item.item)
        .sort((a, b) => {
          const aName = (a.Name || a.ID || '').toLocaleLowerCase();
          const bName = (b.Name || b.ID || '').toLocaleLowerCase();

          if (aName > bName) {
            return 1;
          }

          if (aName < bName) {
            return -1;
          }

          return 0;
        })
        .map((profile) => {
          const spnRaw = getAppSetting<boolean>(profile.Config, 'spn/use');
          const spnOverride: AppVpnOverride =
            typeof spnRaw === 'boolean' ? (spnRaw ? 'vpn' : 'direct') : 'default';
          const effectiveVpn =
            typeof spnRaw === 'boolean' ? spnRaw : this.globalRouteByDefault;

          const splitRaw = getAppSetting<boolean>(profile.Config, 'splittun/use');
          const splitOverride = typeof splitRaw === 'boolean' ? splitRaw : null;
          const effectiveSplitTun =
            this.splitTunEnabled &&
            (splitOverride !== null ? splitOverride : this.globalSplitTunUse);

          return {
            ...profile,
            hasConfigChanges:
              profile.LastEdited > 0 && Object.keys(profile.Config || {}).length > 0,
            selected:
              oldProfiles.get(`${profile.Source}/${profile.ID}`)?.selected ||
              false,
            spnOverride,
            effectiveVpn,
            splitOverride,
            effectiveSplitTun,
          };
        });

      const counts: Record<RouteFilter, number> = {
        all: sortedLocal.length,
        vpn: 0,
        direct: 0,
        custom: 0,
        splittun: 0,
      };
      for (const local of sortedLocal) {
        if (local.effectiveVpn) {
          counts.vpn++;
        } else {
          counts.direct++;
        }
        if (local.spnOverride !== 'default' || local.splitOverride !== null) {
          counts.custom++;
        }
        if (local.effectiveSplitTun) {
          counts.splittun++;
        }
      }
      this.routeCounts = counts;

      const visibleProfiles = sortedLocal.filter((local) => {
        switch (routingFilter) {
          case 'vpn':
            return local.effectiveVpn;
          case 'direct':
            return !local.effectiveVpn;
          case 'custom':
            return local.spnOverride !== 'default' || local.splitOverride !== null;
          case 'splittun':
            return local.effectiveSplitTun;
          default:
            return true;
        }
      });

      this.total = visibleProfiles.length;
      visibleProfiles.forEach((local) => {
        if (activeList.includes(local.Source + '/' + local.ID)) {
          this.runningProfiles.push(local);
        } else if (local.LastEdited >= recentlyUsedThreshold) {
          this.recentlyEdited.push(local);
        }

        // we always add the profile to "All Apps"
        this.profiles.push(local);
      });

      this.changeDetector.markForCheck();
    });
  }

  setRoutingFilter(nextFilter: RouteFilter) {
    this.routingFilter = nextFilter;
    this.onFilterChange.next(nextFilter);
  }

  setGlobalRouteByDefault(value: boolean) {
    this.configService.save('spn/use', value).subscribe({
      next: () => {
        this.actionIndicator.success(
          this.i18n.t('xray.routingSettingSaved'),
          value ? this.i18n.t('xray.routeByDefaultOn') : this.i18n.t('xray.routeByDefaultOff')
        );
      },
      error: (err) => {
        this.actionIndicator.error(
          this.i18n.t('common.error'),
          this.actionIndicator.getErrorMessgae(err)
        );
      },
    });
  }

  setAppVpnMode(profile: LocalAppProfile, mode: AppVpnOverride, event: Event) {
    event.preventDefault();
    event.stopPropagation();

    profile.spnOverride = mode;
    profile.effectiveVpn =
      mode === 'default' ? this.globalRouteByDefault : mode === 'vpn';
    this.changeDetector.markForCheck();

    this.profileService
      .getAppProfile(profile.Source, profile.ID)
      .pipe(
        switchMap((fresh) => {
          const nextConfig: Record<string, any> = { ...(fresh.Config || {}) };
          if (nextConfig['spn'] && typeof nextConfig['spn'] === 'object' && !Array.isArray(nextConfig['spn'])) {
            nextConfig['spn'] = { ...nextConfig['spn'] };
          }
          const updated: AppProfile = {
            ...fresh,
            Config: nextConfig,
          };
          setAppSetting(
            nextConfig,
            'spn/use',
            mode === 'default' ? undefined : mode === 'vpn'
          );
          if (
            mode === 'default' &&
            nextConfig['spn'] &&
            typeof nextConfig['spn'] === 'object' &&
            Object.keys(nextConfig['spn']).length === 0
          ) {
            delete nextConfig['spn'];
          }
          return this.profileService.saveProfile(updated);
        })
      )
      .subscribe({
        error: (err) => {
          this.actionIndicator.error(
            this.i18n.t('common.error'),
            this.actionIndicator.getErrorMessgae(err)
          );
        },
      });
  }

  resetAllToDirect() {
    if (this.resettingRouting) {
      return;
    }
    this.resettingRouting = true;
    this.changeDetector.markForCheck();

    forkJoin([
      this.configService.save('spn/use', false),
      this.xrayService.resetRoutingOverrides(),
    ]).subscribe({
      next: ([, res]) => {
        this.resettingRouting = false;
        this.actionIndicator.success(
          this.i18n.t('xray.resetAllApps'),
          this.i18n.t('xray.resetAllAppsDone', { count: res.updated })
        );
        this.changeDetector.markForCheck();
      },
      error: (err) => {
        this.resettingRouting = false;
        this.actionIndicator.error(
          this.i18n.t('common.error'),
          this.actionIndicator.getErrorMessgae(err)
        );
        this.changeDetector.markForCheck();
      },
    });
  }

  /**
   * @private
   *
   * Used as an ngModelChange callback on the search-input.
   *
   * @param term The search term entered by the user
   */
  searchApps(term: string) {
    this.searchTerm = term;
    this.onSearch.next(term);
  }

  /**
   * @private
   *
   * Opens the create profile dialog
   */
  createProfile() {
    const ref = this.dialog.create(EditProfileDialog, {
      backdrop: true,
      autoclose: false,
    });

    ref.onClose.pipe(filter((action) => action === 'saved')).subscribe(() => {
      // reset the search and reload to make sure the new
      // profile shows up
      this.searchApps('');
    });
  }

  ngOnDestroy() {
    this.subscription.unsubscribe();
  }
}
