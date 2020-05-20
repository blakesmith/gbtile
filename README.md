# GB Tile

A small command line utility to convert PNG images
to [GDBK](http://gbdk.sourceforge.net/) compliant Game Boy
tiles. Tiles are generated as C `unsigned char` arrays.

Takes input like:

![ASCII PNG](https://raw.github.com/blakesmith/gbtile/master/img/ascii.png)

That you can use in your Game Boy programs like:

![Emulator Example](https://raw.github.com/blakesmith/gbtile/master/img/emu_example.png)

## Install

Install [cargo, and rust](https://rustup.rs/)

Then:

```
$ git clone git@github.com:blakesmith/gbtile.git
$ cd gbtile/
$ cargo install --path .
```

The `gbtile` executable will be installed in `$HOME/.cargo/bin/` by default.

## Usage

Find an image that matches the image criteria below, or make your own
in your favorite photo editor, then convert it like so:

```
$ gbtile -i ascii.png -o ascii.tile.h
2020-05-19 21:07:02,154 INFO  [gbtile] File: ascii.png, Tile rows: 14,
columns: 16, unique colors: 2
```

Make sure the tile count and unique colors match your
expectations. The output will be a valid C array like so:

```c
unsigned char ascii[] = {
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
    0x00,0x00,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x00,0x00,0x40,0x40,0x00,0x00,
    ...
    0x10,0x10,0x38,0x38,0x54,0x54,0x50,0x50,0x38,0x38,0x14,0x14,0x54,0x54,0x38,0x38,
};
```

The variable name should match the input file name.

You can now include the tile array in your GBDK Game Boy projects, and
load it using the `set_bkg_data` or `set_sprite_data` C functions.

## Images

For my workflow, I'm using the following image setup:

1. Create an image that has a pixel dimension that's divisible by 8, and no greater than 256x256
2. Use 4 distinct colors. 0xFFFFFF for white, 0x000000 for black. Dark gray, any RGB value between 0xbfbfbf and 0x7f7f7f. For light gray, any RGB color between 0x7f7f7f and 0x3f3f3f.
3. The image will be cut into tiles that are 8x8 pixels wide each.
4. I've been using RGB formatted PNGs, but others should theoretically work.

## License

MIT Licensed.
