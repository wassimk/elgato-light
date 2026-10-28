Output "demo/elgato-light.gif"

Require elgato-light
Require python3

Set Shell "bash"
Set FontFamily "JetBrainsMono Nerd Font"
Set FontSize 16
Set Width 900
Set Height 550
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms

# Sandbox HOME so the CLI reads the fake lights cache, not the real one
Hide
Type "export DEMO_HOME=$(mktemp -d) && mkdir -p $DEMO_HOME/Library/Caches/elgato-light"
Enter
Type "python3 demo/fake_lights.py $DEMO_HOME/Library/Caches/elgato-light/target & sleep 1"
Enter
Type "export HOME=$DEMO_HOME PS1='$ ' && clear"
Enter
Sleep 500ms
Show

Type "elgato-light on --brightness 30 --temperature 4000"
Sleep 500ms
Enter
Sleep 2s

Type "elgato-light brightness -- 15"
Sleep 500ms
Enter
Sleep 2s

Type "elgato-light --light air temperature 5000"
Sleep 500ms
Enter
Sleep 2s

Type "elgato-light status"
Sleep 500ms
Enter
Wait+Screen /Name:\s+Elgato Key Light Air/
Sleep 4s

Type "elgato-light off"
Sleep 500ms
Enter
Sleep 3s

Hide
Type "kill %1; rm -rf $DEMO_HOME"
Enter
