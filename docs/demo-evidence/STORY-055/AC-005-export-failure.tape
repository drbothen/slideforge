Output "AC-005-export-failure.gif"
Output "AC-005-export-failure.webm"

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

Type "slideforge build /tmp/sf-fixtures/deck.sf --format pptx --output-dir /nonexist-root-xyz/dist --no-color 2>&1; echo exit:$?"
Enter
Sleep 5s

Type "ls /nonexist-root-xyz 2>&1 || echo no-partial-output"
Enter
Sleep 2s
