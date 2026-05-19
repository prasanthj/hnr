Output showcase.gif

Set Shell "bash"
Set FontSize 15
Set Width 1600
Set Height 900
Set Theme "Catppuccin Mocha"
Set Padding 0
Set PlaybackSpeed 2.0
Set TypingSpeed 60ms
Set WaitTimeout 15s
Set CursorBlink false

# Launch hnr
Type "/Users/prasanth/work/fun/hnr/target/debug/hnr"
Enter
Sleep 4500ms

# Refresh feed
Type "R"
Sleep 3000ms

# — Top stories — browse slowly so the design reads
Sleep 600ms
Down
Sleep 700ms
Down
Sleep 700ms
Down
Sleep 900ms

# Open comments for story 4
Enter
Sleep 3500ms

# Read the thread
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 600ms

# Expand a comment fully
Enter
Sleep 1000ms

# Collapse the thread
Space
Sleep 800ms

# Scroll more
Down
Sleep 400ms
Down
Sleep 400ms
Down
Sleep 700ms

# Back to stories
Escape
Sleep 700ms

# — New feed —
Type "2"
Sleep 3000ms
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 900ms

# — Best feed —
Type "3"
Sleep 2800ms
Down
Sleep 500ms
Down
Sleep 800ms

# — Ask HN —
Type "4"
Sleep 2800ms
Down
Sleep 500ms
Down
Sleep 600ms
Down
Sleep 900ms

# Open Ask HN comments
Enter
Sleep 3500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 600ms
Escape
Sleep 700ms

# — Show HN —
Type "5"
Sleep 2800ms
Down
Sleep 500ms
Down
Sleep 800ms

# — Back to Top, open reader mode —
Type "1"
Sleep 2500ms
Down
Sleep 500ms
Down
Sleep 700ms

# Reader mode
Type "r"
Sleep 3500ms
Down
Sleep 400ms
Down
Sleep 400ms
Down
Sleep 400ms
Down
Sleep 600ms

# Close reader
Type "q"
Sleep 700ms

# — Search —
Type "?"
Sleep 600ms
Type "rust async"
Sleep 400ms
Enter
Sleep 3000ms
Down
Sleep 500ms
Down
Sleep 800ms

# Back to top
Type "1"
Sleep 2000ms

# Quit
Type "q"
Sleep 400ms
