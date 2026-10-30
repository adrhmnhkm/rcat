# VHS Tape script for rcat terminal demo
# Run: vhs demo.tape to generate demo.gif

Output demo.gif

Set FontSize 16
Set Width 900
Set Height 550
Set Padding 20
Set Theme "Catppuccin Mocha"

Type "rcat --style full Cargo.toml"
Sleep 500ms
Enter
Sleep 2s

Type "rcat -n -r 1:15 src/main.rs"
Sleep 500ms
Enter
Sleep 2s

Type "rcat -X Cargo.toml"
Sleep 500ms
Enter
Sleep 3s

