# NXD Localization Tools for FFT:TIC

Exports and imports text from localization-related NXD files. A simplified version of FF16Tools' database conversion feature.


## Download

Prebuilt binaries are available at the [**Releases**](https://github.com/mmatyas/ffttic-nxdtext/releases).


## Usage

This is a command line application, and works on files already extracted with eg. FF16Tools. Only the files inside `0004.xx.pac` are supported.

- **Export to JSON:**

  `ffttic-nxdtext export your_original_file.nxd --out-json your_output.json`

- **Export to PO:**

  `ffttic-nxdtext export your_original_file.nxd --out-po your_output.po`

- **Import from JSON:**

  `ffttic-nxdtext import your_original_file.nxd --json your_translation.json --out new_nxd_file.nxd`

- **Import from PO:**

  `ffttic-nxdtext import your_original_file.nxd --po your_translation.po --out new_nxd_file.nxd`


## License

This project is available under the GPLv3 license.
