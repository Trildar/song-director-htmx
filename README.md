# Song Sequence Director (HTMX)

Simpler version of https://github.com/Trildar/song-sequence-director built using [htmx](https://htmx.org) and [Tera](https://github.com/Keats/tera) templating rather than Leptos. Small web app for signaling song progression sequence, e.g. verse 1, chorus, verse 2, etc.

## Usage
Run the `song-director-htmx.exe` file to start the server. You can then open the homepage by navigating to `localhost:3000` on a browser on the same computer. You will most likely also need at least one device, such as a phone or tablet, connected to the same local network as the host computer, which the song leader can use. The alternative is to set up an instance of the server that can be accessed on the Internet, which will not be covered here.

The homepage is the director page with buttons for setting the signal. The letter buttons set the signal to the respective letters. The number buttons append the respective numbers to any of the letter signals. The number buttons will not have any effect if there is no current letter signal. The dash `-` clears the signal. The current signal is displayed at the top of the page.

The `/view` page simply displays the current signal. This is mainly intended to be used as an OBS browser source or similar to display the signal, but it can also be used directly in a browser if team members can access the web server from their own devices.

The signal displayed on the director page also synchronises with any changes from other directors, in case you have multiple song leaders.

The intended meaning for each letter is as follows, but you can of course agree on any meaning with your team:

- C: Chorus
- V: Verse
- B: Bridge
- P: Pre-chorus
- W: Worship (Instruments only)
- E: Ending/Last line
- X: Stop/Finish
- R: Repeat/Play on

## Attributions

The music notes used for the favicon were obtained from <a href="https://www.flaticon.com/free-icons/music" title="music icons">Freepik on Flaticon</a>

