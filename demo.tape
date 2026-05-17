Output demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 800
Set Theme "Catppuccin Mocha"
Set Padding 20
Set PlaybackSpeed 1

# Launch hnr
Type "/Users/prasanth/work/fun/hnr/target/release/hnr"
Enter
Sleep 4s

# Browse top stories
Sleep 500ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Open comments
Enter
Sleep 3s

# Scroll comments
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Open comment detail overlay
Enter
Sleep 1500ms

# Close overlay
Escape
Sleep 500ms

# Go back to stories
Escape
Sleep 500ms

# Search
Type "?"
Sleep 500ms
Type "rust"
Sleep 400ms
Enter
Sleep 3s

# Browse search results
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Bookmark one
Type "b"
Sleep 800ms

# Switch to bookmarks
Type "7"
Sleep 1500ms

# Quit
Type "q"
Sleep 500ms
