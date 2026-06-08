Output "AC-012-exit-code-precedence.gif"
Output "AC-012-exit-code-precedence.webm"

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
Type "rm -rf /tmp/sf-dist-mixed"
Enter
Sleep 300ms
Show

Type "cat /tmp/sf-fixtures/mixed_errors.sf"
Enter
Sleep 1s

Type "slideforge build /tmp/sf-fixtures/mixed_errors.sf --output-dir /tmp/sf-dist-mixed --no-color 2>&1; echo exit:$?"
Enter
Sleep 5s
