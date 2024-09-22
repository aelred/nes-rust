A NES emulator written in Rust, go to [nes.ael.red](https://nes.ael.red) to see it in action!

# Quickstart

To run locally, [install Just](https://just.systems/man/en/packages.html), get a NES ROM file and run:

```bash
just run <path to .nes ROM file>
```

If you need a NES ROM, try the [Alwa's Awakening demo](https://eldenpixels.itch.io/alwas-awakening-the-8-bit-edition#:~:text=944%20kB-,Download%20demo,-Download).

To run the web version, run:

```bash
just serve
```

And navigate to [http://localhost:8080](http://localhost:8080).

# Runtimes

The emulator supports two "runtimes": [SDL](#sdl) and [web](#web).

- SDL: Runs locally on your computer using SDL 2.
- Web: Runs in the browser using WASM, configured in [./web](./web).

# Deployment

The emulator is deployed to the web with GitHub pages in [deploy.yml](./.github/workflows/deploy.yml), using infrastructure provisioned with terraform in [./deploy/infrastructure](./deploy/infrastructure). It's available at [nes.ael.red](https://nes.ael.red).