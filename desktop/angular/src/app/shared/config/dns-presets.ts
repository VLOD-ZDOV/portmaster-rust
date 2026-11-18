import { QuickSetting } from '@safing/portmaster-api';

export const DNS_PRESETS: QuickSetting<string[]>[] = [
  {
    Name: 'qs.dns.allDot',
    Value: [
      'dot://cloudflare-dns.com?ip=1.1.1.1&name=Cloudflare',
      'dot://dns.quad9.net?ip=9.9.9.9&name=Quad9&blockedif=empty',
      'dot://dns.adguard-dns.com?ip=94.140.14.14&name=AdGuard&blockedif=zeroip',
      'dot://dns.google?ip=8.8.8.8&name=Google',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.allDoh',
    Value: [
      'https://cloudflare-dns.com/dns-query?ip=1.1.1.1&name=Cloudflare',
      'https://dns.quad9.net/dns-query?ip=9.9.9.9&name=Quad9&blockedif=empty',
      'https://dns.adguard-dns.com/dns-query?ip=94.140.14.14&name=AdGuard&blockedif=zeroip',
      'https://dns.google/dns-query?ip=8.8.8.8&name=Google',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.cloudflareDot',
    Value: [
      'dot://cloudflare-dns.com?ip=1.1.1.1&name=Cloudflare',
      'dot://cloudflare-dns.com?ip=1.0.0.1&name=Cloudflare',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.cloudflareDoh',
    Value: [
      'https://cloudflare-dns.com/dns-query?ip=1.1.1.1&name=Cloudflare',
      'https://cloudflare-dns.com/dns-query?ip=1.0.0.1&name=Cloudflare',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.cloudflareMalwareDot',
    Value: [
      'dot://cloudflare-dns.com?ip=1.1.1.2&name=Cloudflare&blockedif=zeroip',
      'dot://cloudflare-dns.com?ip=1.0.0.2&name=Cloudflare&blockedif=zeroip',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.cloudflareMalwareDoh',
    Value: [
      'https://security.cloudflare-dns.com/dns-query?ip=1.1.1.2&name=Cloudflare&blockedif=zeroip',
      'https://security.cloudflare-dns.com/dns-query?ip=1.0.0.2&name=Cloudflare&blockedif=zeroip',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.quad9Dot',
    Value: [
      'dot://dns.quad9.net?ip=9.9.9.9&name=Quad9&blockedif=empty',
      'dot://dns.quad9.net?ip=149.112.112.112&name=Quad9&blockedif=empty',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.quad9Doh',
    Value: [
      'https://dns.quad9.net/dns-query?ip=9.9.9.9&name=Quad9&blockedif=empty',
      'https://dns.quad9.net/dns-query?ip=149.112.112.112&name=Quad9&blockedif=empty',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.adguardDot',
    Value: [
      'dot://dns.adguard-dns.com?ip=94.140.14.14&name=AdGuard&blockedif=zeroip',
      'dot://dns.adguard-dns.com?ip=94.140.15.15&name=AdGuard&blockedif=zeroip',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.adguardDoh',
    Value: [
      'https://dns.adguard-dns.com/dns-query?ip=94.140.14.14&name=AdGuard&blockedif=zeroip',
      'https://dns.adguard-dns.com/dns-query?ip=94.140.15.15&name=AdGuard&blockedif=zeroip',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.googleDot',
    Value: [
      'dot://dns.google?ip=8.8.8.8&name=Google',
      'dot://dns.google?ip=8.8.4.4&name=Google',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.googleDoh',
    Value: [
      'https://dns.google/dns-query?ip=8.8.8.8&name=Google',
      'https://dns.google/dns-query?ip=8.8.4.4&name=Google',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.mullvadDot',
    Value: [
      'dot://dns.mullvad.net?ip=194.242.2.3&name=Mullvad',
      'dot://dns.mullvad.net?ip=194.242.2.4&name=Mullvad',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.mullvadDoh',
    Value: [
      'https://dns.mullvad.net/dns-query?ip=194.242.2.3&name=Mullvad',
      'https://dns.mullvad.net/dns-query?ip=194.242.2.4&name=Mullvad',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.controldDot',
    Value: [
      'dot://p2.freedns.controld.com?ip=76.76.2.2&name=ControlD',
      'dot://p2.freedns.controld.com?ip=76.76.10.2&name=ControlD',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.controldDoh',
    Value: [
      'https://freedns.controld.com/p2?ip=76.76.2.2&name=ControlD',
      'https://freedns.controld.com/p2?ip=76.76.10.2&name=ControlD',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.appliedPrivacyDot',
    Value: [
      'dot://dot1.applied-privacy.net?ip=146.255.56.98&name=AppliedPrivacy',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.appliedPrivacyDoh',
    Value: [
      'https://doh.applied-privacy.net/query?ip=146.255.56.98&name=AppliedPrivacy',
    ],
    Action: 'replace',
  },
  {
    Name: 'qs.dns.cloudflareFallback',
    Value: [
      'dot://cloudflare-dns.com?ip=1.1.1.1&name=Cloudflare&blockedif=zeroip',
      'dot://cloudflare-dns.com?ip=1.0.0.1&name=Cloudflare&blockedif=zeroip',
    ],
    Action: 'merge-bottom',
  },
];
