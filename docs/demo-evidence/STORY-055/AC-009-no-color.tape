Output "AC-009-no-color.gif"
Output "AC-009-no-color.webm"

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

Type "slideforge build /tmp/sf-fixtures/parse_error.sf --output-dir /tmp/sf-nc1 --no-color 2>&1; echo exit:$?"
Enter
Sleep 5s

Hide
Type "clear"
Enter
Sleep 300ms
Show

Type "slideforge build /tmp/sf-fixtures/parse_error.sf --output-dir /tmp/sf-nc2 --no-color 2>&1 | cat -v | grep -c ESC && echo ANSI-present || echo CLEAN-no-ANSI"
Enter
Sleep 5s
