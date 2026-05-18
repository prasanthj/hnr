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

# Launch
Type "hnr"
Enter
Sleep 4s

# Browse — seen stories are dimmed automatically
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 600ms

# Reader mode — structured rendering unique to hnr
Type "r"
Sleep 6s

# Jump between article sections with n
Type "n"
Sleep 1200ms
Type "n"
Sleep 1200ms

# Page down
Type "d"
Sleep 900ms

# Close reader
Escape
Sleep 700ms

# Fetch threaded comments
Enter
Sleep 4500ms

# Navigate comment tree
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Collapse a thread
Type " "
Sleep 800ms
Type " "
Sleep 600ms

# View author profile — press p
Type "p"
Sleep 2500ms
Escape
Sleep 700ms

# Switch back to stories
Tab
Sleep 600ms

# Bookmark the story
Type "b"
Sleep 700ms

# Browse the bookmark feed
Type "7"
Sleep 1500ms

# Back to top feed
Type "1"
Sleep 900ms

# Login modal — shows hnr supports voting and replies
Type "l"
Sleep 1800ms
Escape
Sleep 500ms

# Full-text search via Algolia
Type "?"
Sleep 500ms
Type "rust async"
Sleep 400ms
Enter
Sleep 4s

# Browse results
Type "j"
Sleep 350ms
Type "j"
Sleep 350ms
Type "j"
Sleep 350ms

# Quit
Type "q"
Sleep 500ms
