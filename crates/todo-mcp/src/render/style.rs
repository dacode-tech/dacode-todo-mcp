//! The pages' stylesheet (U58's split out of `page.rs`, paying that
//! file's recorded file-size debt): the shared house style every
//! render carries in its `<style>` block — the repo page's and the
//! epic page's alike, so the two cannot drift apart. The other style
//! blocks stay with their sections: the sidebar's, the activity
//! panel's and the rewards section's rules live in their own
//! modules, and the epic page's current-epic marker rule is
//! page-local (`epic`'s `EPIC_CSS`, appended after this block).

/// The page's stylesheet — the Python tool's CSS ported verbatim plus
/// the switcher's rules; the epic page reuses it unchanged (the
/// shared house style, so the two pages cannot drift apart).
pub(super) const CSS: &str = r#":root { color-scheme: light dark; }
body { font: 14px/1.5 -apple-system, "Segoe UI", Roboto, sans-serif; margin: 0;
       padding: 24px 28px 60px; color: #16181d; background: #fbfbfa; }
h1 { font-size: 20px; margin: 0 0 4px; }
h2 { font-size: 14px; text-transform: uppercase; letter-spacing: .08em;
     color: #6b7280; margin: 30px 0 10px; }
.sub { color: #6b7280; font-size: 12.5px; margin-bottom: 18px; }
.switch { display: flex; flex-wrap: wrap; gap: 6px; margin: 0 0 16px; }
.switch a, .switch span { border: 1px solid #e3e3e0; border-radius: 6px; padding: 3px 10px;
                          font-size: 12px; background: #fff; color: #3f444c;
                          text-decoration: none; }
.switch .cur { background: #e6f4ec; border-color: #bfe0cd; color: #1f6f43; font-weight: 600; }
.cards { display: flex; flex-wrap: wrap; gap: 10px; }
.card { border: 1px solid #e3e3e0; border-radius: 10px; padding: 10px 14px;
        background: #fff; min-width: 148px; }
.card b { display: block; font-size: 21px; font-variant-numeric: tabular-nums; }
.card i { font-style: normal; font-size: 11px; text-transform: uppercase;
          letter-spacing: .07em; color: #6b7280; }
.card small { display: block; color: #6b7280; font-size: 11.5px; margin-top: 4px;
              max-width: 210px; }
.card .p { float: right; font-size: 13px; color: #2f6f4f; font-weight: 600; }
.msrow { display: flex; gap: 8px; flex-wrap: wrap; }
.ms { border: 1px solid #e3e3e0; border-radius: 8px; padding: 6px 12px; background: #fff;
      font-size: 12.5px; }
.ms b { font-size: 13px; }
.ms span { display: block; color: #6b7280; font-size: 11px; }
.msdone { background: #e6f4ec; border-color: #bfe0cd; }
.msdone span { color: #1f6f43; }
.scroll { overflow-x: auto; border: 1px solid #e3e3e0; border-radius: 10px; background: #fff; }
svg text { font-family: inherit; }
.lane { font-size: 11px; fill: #6b7280; text-transform: uppercase; letter-spacing: .06em; }
.laneline { stroke: #efefec; stroke-width: 1; }
.nid { font-size: 11.5px; font-weight: 700; }
.nlabel { font-size: 10.5px; fill: #3f444c; }
.node { cursor: pointer; }
.node:hover rect { stroke-width: 2.6; }
.edge { fill: none; stroke: #2f6f4f; stroke-width: 1.3; opacity: .55; }
table { border-collapse: collapse; width: 100%; background: #fff; font-size: 12.5px; }
th, td { text-align: left; padding: 6px 9px; border-bottom: 1px solid #eee; vertical-align: top; }
th { font-size: 11px; text-transform: uppercase; letter-spacing: .06em; color: #6b7280; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.st-closed { color: #1f6f43; } .st-open { color: #8a5a00; }
.st-in_work { color: #8a5a00; } .st-blocked { color: #9b1c1c; font-weight: 700; }
.st-signed { color: #1f6f43; }
#detail { border: 1px solid #e3e3e0; border-left: 4px solid #2f6f4f; border-radius: 8px;
          background: #fff; padding: 10px 14px; margin-top: 10px; display: none;
          font-size: 12.5px; }
#detail b { font-size: 13.5px; }
.legend { font-size: 11.5px; color: #6b7280; margin: 8px 0 0; }
.legend span { display: inline-block; margin-right: 14px; }
.sw { display: inline-block; width: 9px; height: 9px; border-radius: 2px; margin-right: 4px; }
footer { margin-top: 34px; color: #6b7280; font-size: 11.5px; border-top: 1px solid #eee;
         padding-top: 10px; }
"#;
