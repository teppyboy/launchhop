# LaunchHop

Launch a process but in a complicated way (Windows only)

## Features

+ Launch a process with a different executable

## Usage

1. Download the latest release (the `-release` file is the one you want)
2. Changge the `config.toml` to your needs
3. Run `launchhop.exe`
> If the application doesn't launch, try launching with administrator

## Technical Details

launchhop will launch the `launcher` process and inject a DLL into it. The DLL will then launch the `target` process 
and exit or wait for the `target` process to exit. The `launcher` process will then exit.

E.g. If you use `launchhop.exe` and configured correctly to launch `StarRail.exe` it'll be something like this:

`launchhop.exe` -> `launcher.exe` -> `StarRail.exe` (then Star Rail will see `launcher.exe` as the *parent process*)

This can be used to circumvent some checks that check the parent process, like Star Rail does and upload it to their
logging servers.

## License

[MIT](./LICENSE)
