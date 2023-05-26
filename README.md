# Yama

Yama will automatically organize your local animes, displaying them in a neatly manner and keeping watch where you left out watching!

![Yama titles screenshot](./docs/readme_resources/yama_titles_screenshot.png)

![Yama episodes screenshot](./docs/readme_resources/yama_episodes_screenshot.png)

<br/><br/>

## Index:
- [Usage](#usage)
- [Config](#config-paths)
- [Dependencies](#dependencies)

<br/><br/>

## Usage:

From now it will default to read all titles in a folder called 'series' located alongside the executable (it will be generated if not found), but it can be changed in the config menu (pic below).

<p align="center">
    <img src="./docs/readme_resources/config_screenshot.png" alt="Config screenshot">
</p>

Then, yama will search with the [Anilist API](https://anilist.gitbook.io/anilist-apiv2-docs/) for information about the anime based on the name of the folder, so if for some reason it cannot found info about the anime try renaming it to something more close to the original title.

Once entered in a title, yama will proceed to generate the metadata for each episode, this will probably take a bit of time depending on the amount of episodes and its size, but its only the first time, after that yama will cache the results in a hidden folder inside the title called '.metadata'.

Yama will also keep saved at what time you left an episode, and if you finish it watching it will mark it as such, so you can always remember where you left off an anime.

You will also have these three functions for easy of use:
<p align="center">
    <img src="./docs/readme_resources/episodes_function_screenshot.png" alt="Episodes funciton screenshot">
</p>

- Refresh: Will refresh the generate metadata for that title in case you have added more episodes and they are not visible in the program.
- Mark as watched: To quickly mark an episode as watched.
- Mark all previous as watched: To mark all episodes before the current one selected as watched.

<br/><br/>

## Config paths:
- Linuxs: $HOME/.config/yama
- Windows: %appdata%/Roaming/yama

> If any errors occurred while using yama you can generate a new issue with the output of the last log file located in the config folder.

<br/><br/>

## Dependencies:

Yama requires **ffmpeg** and **mpv** to work, for linux users you can install it with your packet manager, but for windows users there is a dependencies.ps1 script in the [Release](https://github.com/sad-ko/yama/releases) zip, it will download both programs and add it to the users Path.

<br/><br/>
<br/><br/>

<p align="center">
    <img src="./docs/readme_resources/about_screenshot.png" alt="About">
    <img height=250 src="./res/yama.png" alt="Yama">
</p>
