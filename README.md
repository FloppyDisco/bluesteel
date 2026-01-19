# Bluesteel Software co
### website

the static site lives in /site/
the cloudflare pages serves the /site/ dir with no build commands

to build the wasm files once run `./build.sh`

bacon is a watch tool that can build the lib on changes to the code for hot reloading in the dev environment using `bacon build`

the wasm binaries are available on the window at `.wasm`

to serve the dev env run `./dev.sh`