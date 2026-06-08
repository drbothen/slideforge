Output "AC-004-eval-warn-only.gif"
Output "AC-004-eval-warn-only.webm"

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
Type "rm -rf /tmp/sf-dist-warn-no /tmp/sf-dist-warn-yes"
Enter
Sleep 300ms
Show

Type "slideforge build /tmp/sf-fixtures/eval_error.sf --format pptx --output-dir /tmp/sf-dist-warn-no --no-color 2>&1; echo exit:$?"
Enter
Sleep 6s

Type "ls /tmp/sf-dist-warn-no 2>&1 || echo no-output-dir"
Enter
Sleep 1s

Hide
Type "clear"
Enter
Sleep 300ms
Show

Type "slideforge build /tmp/sf-fixtures/eval_error.sf --format pptx --output-dir /tmp/sf-dist-warn-yes --warn-only --no-color 2>&1; echo exit:$?"
Enter
Sleep 6s

Type "ls /tmp/sf-dist-warn-yes/"
Enter
Sleep 2s
