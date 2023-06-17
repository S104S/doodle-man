# Method Dev Assignment - Twitch Bot #

## Overview ##
---
Create an automated [Twitch](https://dev.twitch.tv/docs/irc) chat bot console application that can be run from a command line interface (CLI).


## Requirements
---
The bot application should be able to:
* Console output all interactions - legibly formatted, with timestamps.
* Connect to Twitch IRC over SSL.
* Avoid premature disconnections by handling Twitch courier ping / pong requests.
* Join a channel.
* Read a channel.
* Write to a channel - reply to a user-issued string command within a channel (e.g. !YOUR_COMMAND_NAME).
	* BRYAN SPECIFICALLY: Reply to the "!chucknorris" command by dynamically returning a random fact about Chuck Norris using the [Chuck Norris API](https://api.chucknorris.io).


## Caveats ##
---
* Perferrably, the application should be written in [Rust](https://www.rust-lang.org/) or [Go lang](https://golang.org/pkg/), using the standard library.
	* If time is a limiting factor, using a platform of your choice is permitted upon request.
* If at all possible, the application should be written without third-party module dependencies.
* All interactions should be asynchronous.
* The application should account for Twitch API rate limits.
* The application should not exit prematurely.


## Contact ##
---
If you have any questions, feel free to reach out to the following folks:
* [Nicholas Cassidy](mailto:nick@method.dev)
* [Doug Rayburn](mailto:doug@method.dev)
* [Jake Wilson](mailto:jake@method.dev)
* [Charlie Winski](mailto:charlie@method.dev)