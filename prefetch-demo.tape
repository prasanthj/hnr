Output prefetch-demo.gif

Set Shell "zsh"
Set FontSize 15
Set Width 1600
Set Height 900
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 80ms

Hide
Type "export PATH=$HOME/.cargo/bin:$PATH"
Enter
Sleep 200ms
Type "cp /Users/prasanth/work/fun/hnr/target/debug/hnr ~/.cargo/bin/hnr"
Enter
Sleep 500ms
Show

# Launch
Type "hnr"
Enter
Sleep 4s

# Sit on first story — Fetching comments shimmer appears immediately
# 3s debounce fires, comments auto-arrive and pane switches
Sleep 8s

# Back to stories
Tab
Sleep 800ms

# Refresh feed — shows Refreshing shimmer while loading
Type "R"
Sleep 4s

# Quit
Type "q"
Sleep 300ms
