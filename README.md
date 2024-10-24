<h1 align="center">
    <br>
    Funboy Discord Bot
    <br>
</h1>

# Overview

The Funboy discord bot is designed primarly to generate semi-randomized text from a user created and modified database of templates and substitutes. A template is simply a label that maps to a list of substitutes that will be randomly selected from when generating text. When generating text templates are marked by starting with a valid template charater (', ^, or `) such as ^noun. This markation lets the **/generate** command know which text should be substituted. As an example a user might create a template called "animal" and store substitutes such as "cat, dog, bat" and then use the generate command to produce randomized output by entering "My favorite animal is ^animal." which will randomly replace ^animal with any substitutes present in the template.
Example output: **My favorite animal is dog**
The bot also has it's own scripting language (FSL - Funboy Scripting Language) that the generate command can interpret. Use the /fsl_help command to learn how to use it. 
In addition, the bot is capable of playing music and sounds from the web by using the **/join_voice** command to enter a voice channel and **/play_track** to play a url or search for a track name and play it. Tracks can individually be manipulated by using **/show_tracks** to get a list of currently playing tracks and track controls.

# Installation and Usage

This bot is not publically hosted so in order to use it you must install and use cargo to build it from source and host it yourself with a valid discord token. Once you've installed cargo and gotten a discord token you can use the terminal to build the source code with **cargo build --release** and then run the bot by setting the discord token environment variable with **DISCORD_TOKEN=your_token_here** and then run the generated build file in **/target/release/funboy**
The bot optionally uses an IMGUR_CLIENT_ID variable for the **/search_image** command but everything else will still work without it.
Once the bot is set up use /help to get a list of bot commands and descriptions of what each command does.

# License

This bot is released under the MIT license so you may use it however you wish.
