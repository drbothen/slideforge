Output "AC-003-eval-error-strict.gif"
Output "AC-003-eval-error-strict.webm"

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
Type "rm -rf /tmp/sf-dist-eval"
Enter
Sleep 300ms
Show

Type "cat /tmp/sf-fixtures/eval_error.sf"
Enter
Sleep 1s

Type "slideforge build /tmp/sf-fixtures/eval_error.sf --output-dir /tmp/sf-dist-eval --no-color 2>&1; echo exit:$?"
Enter
Sleep 5s

Type "ls /tmp/sf-dist-eval 2>&1 || echo no-output-dir"
Enter
Sleep 2s
