/* STM32F103C8 Blue Pill (conservative profile) */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM   : ORIGIN = 0x20000000, LENGTH = 20K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
__STACK_START = _stack_start;