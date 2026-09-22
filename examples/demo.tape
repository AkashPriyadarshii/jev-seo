Output "demo.gif", 640x400, 30fps. Renders jev-seo terminal runs.
Set Shell "bash"
Set FontSize 13
Set Padding 12
Set Theme "Dracula"

Type "jev-seo crawl https://jevseo.vercel.app/ --max-pages 8" Sleep 1s Enter
Wait +2s
Type "jev-seo llms jevseo.vercel.app" Sleep 1s Enter
Wait +2s
Type "jev-seo audit docs/ --min-pass 40" Sleep 1s Enter
Wait +3s
