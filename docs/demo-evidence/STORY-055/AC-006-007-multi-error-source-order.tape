Output "AC-006-007-multi-error-source-order.gif"
Output "AC-006-007-multi-error-source-order.webm"

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
Type "rm -rf /tmp/sf-dist-multi"
Enter
Sleep 300ms
Show

Type "cat /tmp/sf-fixtures/multi_error.sf"
Enter
Sleep 1s

Type "slideforge build /tmp/sf-fixtures/multi_error.sf --output-dir /tmp/sf-dist-multi --no-color 2>&1; echo exit:$?"
Enter
Sleep 5s

Type "ls /tmp/sf-dist-multi 2>&1 || echo no-output-dir"
Enter
Sleep 2s
