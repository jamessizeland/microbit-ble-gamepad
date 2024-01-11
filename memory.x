MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  MBR                               : ORIGIN = 0x00000000, LENGTH = 4K
  SOFTDEVICE                        : ORIGIN = 0x00001000, LENGTH = 114688
  FLASH                             : ORIGIN = 0x00027000, LENGTH = 405504
  RAM                               : ORIGIN = 0x20007b08, LENGTH = 86104
}
