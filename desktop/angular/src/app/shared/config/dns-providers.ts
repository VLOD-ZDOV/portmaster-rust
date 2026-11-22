export type DnsProtocol = 'dot' | 'doh';

export interface DnsProvider {
  id: string;
  name: string;
  /** i18n key of the short description. */
  desc: string;
  /** TLS hostname used for DoT and DoH certificate checks. */
  host: string;
  ips: string[];
  /** DoH path on `host`. */
  dohPath: string;
  /** Upstream block detection, see resolver `blockedif` parameter. */
  blockedIf?: 'zeroip' | 'empty' | 'refused';
  /** Overrides the DoT hostname when it differs from the DoH one. */
  dotHost?: string;
}

export const DNS_PROVIDERS: DnsProvider[] = [
  { id: 'cloudflare', name: 'Cloudflare', desc: 'dnsPicker.desc.plain', host: 'cloudflare-dns.com', ips: ['1.1.1.1', '1.0.0.1'], dohPath: '/dns-query' },
  { id: 'cloudflare-security', name: 'Cloudflare Security', desc: 'dnsPicker.desc.malware', host: 'security.cloudflare-dns.com', ips: ['1.1.1.2', '1.0.0.2'], dohPath: '/dns-query', blockedIf: 'zeroip' },
  { id: 'cloudflare-family', name: 'Cloudflare Family', desc: 'dnsPicker.desc.family', host: 'family.cloudflare-dns.com', ips: ['1.1.1.3', '1.0.0.3'], dohPath: '/dns-query', blockedIf: 'zeroip' },
  { id: 'quad9', name: 'Quad9', desc: 'dnsPicker.desc.malware', host: 'dns.quad9.net', ips: ['9.9.9.9', '149.112.112.112'], dohPath: '/dns-query', blockedIf: 'empty' },
  { id: 'quad9-unfiltered', name: 'Quad9 Unfiltered', desc: 'dnsPicker.desc.plain', host: 'dns10.quad9.net', ips: ['9.9.9.10', '149.112.112.10'], dohPath: '/dns-query' },
  { id: 'google', name: 'Google', desc: 'dnsPicker.desc.plain', host: 'dns.google', ips: ['8.8.8.8', '8.8.4.4'], dohPath: '/dns-query' },
  { id: 'adguard', name: 'AdGuard', desc: 'dnsPicker.desc.ads', host: 'dns.adguard-dns.com', ips: ['94.140.14.14', '94.140.15.15'], dohPath: '/dns-query', blockedIf: 'zeroip' },
  { id: 'adguard-family', name: 'AdGuard Family', desc: 'dnsPicker.desc.family', host: 'family.adguard-dns.com', ips: ['94.140.14.15', '94.140.15.16'], dohPath: '/dns-query', blockedIf: 'zeroip' },
  { id: 'adguard-unfiltered', name: 'AdGuard Unfiltered', desc: 'dnsPicker.desc.plain', host: 'unfiltered.adguard-dns.com', ips: ['94.140.14.140', '94.140.14.141'], dohPath: '/dns-query' },
  { id: 'mullvad', name: 'Mullvad', desc: 'dnsPicker.desc.plain', host: 'dns.mullvad.net', ips: ['194.242.2.2'], dohPath: '/dns-query' },
  { id: 'mullvad-adblock', name: 'Mullvad Adblock', desc: 'dnsPicker.desc.ads', host: 'adblock.dns.mullvad.net', ips: ['194.242.2.3'], dohPath: '/dns-query' },
  { id: 'controld', name: 'Control D', desc: 'dnsPicker.desc.plain', host: 'freedns.controld.com', dotHost: 'p0.freedns.controld.com', ips: ['76.76.2.0', '76.76.10.0'], dohPath: '/p0' },
  { id: 'controld-ads', name: 'Control D Ads', desc: 'dnsPicker.desc.ads', host: 'freedns.controld.com', dotHost: 'p2.freedns.controld.com', ips: ['76.76.2.2', '76.76.10.2'], dohPath: '/p2', blockedIf: 'zeroip' },
  { id: 'nextdns', name: 'NextDNS', desc: 'dnsPicker.desc.plain', host: 'dns.nextdns.io', ips: ['45.90.28.0', '45.90.30.0'], dohPath: '/' },
  { id: 'dnssb', name: 'DNS.SB', desc: 'dnsPicker.desc.plain', host: 'dot.sb', ips: ['185.222.222.222', '45.11.45.11'], dohPath: '/dns-query' },
  { id: 'applied-privacy', name: 'Applied Privacy', desc: 'dnsPicker.desc.plain', host: 'doh.applied-privacy.net', dotHost: 'dot1.applied-privacy.net', ips: ['146.255.56.98'], dohPath: '/query' },
];

/** Builds the resolver URLs (one per IP) in the format the core parses. */
export function dnsProviderUrls(p: DnsProvider, proto: DnsProtocol): string[] {
  const params = (ip: string) => {
    let q = `ip=${ip}&name=${encodeURIComponent(p.name)}`;
    if (p.blockedIf) {
      q += `&blockedif=${p.blockedIf}`;
    }
    return q;
  };
  return p.ips.map(ip =>
    proto === 'dot'
      ? `dot://${p.dotHost ?? p.host}?${params(ip)}`
      : `https://${p.host}${p.dohPath}?${params(ip)}`,
  );
}

/** Returns true if any configured URL points to this provider. */
export function dnsProviderSelected(p: DnsProvider, current: string[]): boolean {
  const hosts = [p.host, p.dotHost].filter(Boolean) as string[];
  return current.some(url => {
    try {
      const u = new URL(url.replace(/^dot:/, 'https:'));
      const ip = u.searchParams.get('ip');
      return hosts.includes(u.hostname) && (!ip || p.ips.includes(ip));
    } catch {
      return false;
    }
  });
}
