Output "AC-011-tracing-spans.gif"
Output "AC-011-tracing-spans.webm"

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "rm -rf /tmp/sf-dist-trace"
Enter
Sleep 300ms
Show

Type "RUST_LOG=info slideforge build /tmp/sf-fixtures/deck.sf --format pptx --output-dir /tmp/sf-dist-trace 2>&1 | grep INFO | grep stage"
Enter
Sleep 8s
