Output demo.gif

Set Shell "zsh"
Set FontSize 15
Set Width 1600
Set Height 900
Set Theme "Catppuccin Mocha"
Set Padding 24
Set PlaybackSpeed 1
Set TypingSpeed 80ms

Hide
Type "export PATH=$HOME/.cargo/bin:$PATH"
Enter
Sleep 500ms
Show

# Show install command
Type "cargo install hnr"
Sleep 1s
Enter
Sleep 6s

# Launch
Type "hnr"
Enter
Sleep 5s

# Browse top stories
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 600ms

# Reader mode
Type "r"
Sleep 5s

# Scroll article
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Close reader
Escape
Sleep 800ms

# Fetch comments
Enter
Sleep 4000ms

# Navigate comments
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 500ms

# Collapse a thread
Type " "
Sleep 800ms

# Expand it back
Type " "
Sleep 600ms

# Expand comment inline
Enter
Sleep 2000ms

# Collapse it
Enter
Sleep 600ms

# Switch back to stories
Tab
Sleep 800ms

# Algolia search
Type "?"
Sleep 700ms
Type "golang"
Sleep 500ms
Enter
Sleep 4s

# Browse results
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Quit
Type "q"
Sleep 500ms
