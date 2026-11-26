import { QuickSetting } from '@safing/portmaster-api';

/** Built-in Quick Settings for Xray internal routing rules (`xray/rules`, requires geoip.dat / geosite.dat for geo categories). */
export const XRAY_RULE_PRESETS: QuickSetting<string[]>[] = [
  {
    Name: 'qs.xray.ruDirect',
    Action: 'merge-bottom',
    Value: [
      '- LAN',
      '- Localhost',
      '- geosite:category-ru',
      '- geoip:ru',
      '- .ru',
      '- .su',
      '- .рф',
    ],
  },
  {
    Name: 'qs.xray.adsBlock',
    Action: 'merge-bottom',
    Value: [
      '- geosite:category-ads-all',
    ],
  },
  {
    Name: 'qs.xray.youtubeDiscord',
    Action: 'merge-bottom',
    Value: [
      '+ geosite:youtube',
      '+ geosite:discord',
      '+ geosite:telegram',
      '+ geosite:openai',
    ],
  },
  {
    Name: 'qs.xray.socialMedia',
    Action: 'merge-bottom',
    Value: [
      '+ geosite:instagram',
      '+ geosite:twitter',
      '+ geosite:twitch',
      '+ geosite:spotify',
    ],
  },
];

/** Built-in Quick Settings for firewall VPN policy (`spn/usagePolicy`, uses Portmaster's built-in MaxMind GeoIP & domain rules). */
export const FIREWALL_VPN_POLICY_PRESETS: QuickSetting<string[]>[] = [
  {
    Name: 'qs.policy.ruDirect',
    Action: 'merge-bottom',
    Value: [
      '- LAN',
      '- Localhost',
      '- RU',
      '- .ru',
      '- .su',
      '- .рф',
    ],
  },
  {
    Name: 'qs.policy.youtube',
    Action: 'merge-bottom',
    Value: [
      '+ .youtube.com',
      '+ .googlevideo.com',
      '+ .ytimg.com',
      '+ .youtu.be',
    ],
  },
  {
    Name: 'qs.policy.discord',
    Action: 'merge-bottom',
    Value: [
      '+ .discord.com',
      '+ .discordapp.com',
      '+ .discordapp.net',
      '+ .discord.gg',
      '+ .discord.media',
    ],
  },
  {
    Name: 'qs.policy.telegram',
    Action: 'merge-bottom',
    Value: [
      '+ .telegram.org',
      '+ .t.me',
      '+ .tdesktop.com',
      '+ AS62041',
      '+ AS62014',
      '+ AS59930',
      '+ AS44907',
    ],
  },
  {
    Name: 'qs.policy.ai',
    Action: 'merge-bottom',
    Value: [
      '+ .openai.com',
      '+ .chatgpt.com',
      '+ .anthropic.com',
      '+ .claude.ai',
    ],
  },
];
