Output "AC-001-010-success-all-formats.gif"
Output "AC-001-010-success-all-formats.webm"

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
Type "rm -rf /tmp/sf-dist-all /tmp/sf-dist-fmt"
Enter
Sleep 400ms
Show

Type "cat /tmp/sf-fixtures/deck.sf"
Enter
Sleep 1s

Type "slideforge build /tmp/sf-fixtures/deck.sf --output-dir /tmp/sf-dist-all 2>/dev/null; echo exit:$?"
Enter
Sleep 8s

Type "ls /tmp/sf-dist-all/"
Enter
Sleep 2s

Hide
Type "clear"
Enter
Sleep 300ms
Show

Type "slideforge build /tmp/sf-fixtures/deck.sf --format pptx,html --output-dir /tmp/sf-dist-fmt 2>/dev/null; echo exit:$?"
Enter
Sleep 8s

Type "ls /tmp/sf-dist-fmt/"
Enter
Sleep 2s
