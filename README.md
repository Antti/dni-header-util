# dni-header-util

Display and manipulate Netgear DNI firmware header

## Usage

Reading header:

```
# dni-header-util show FIRMWARE_FILE.img

Checksum match: 0x92
DNI Header:
  device:RBR50
  version:2.5.1.16
  region:
  hd_id:29765352+0+4000+512+2x2+2x2+4x4

```

Writing header:
The tool by default will copy all other header fields and will only replace the given one:

```
# dni-header-util set FIRMWARE_FILE.img OUTPUT_FIRMWARE_FILE.img --key device --value RBR50
```

