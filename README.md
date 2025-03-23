# DS2S-HEAP-X

### Dark Souls II Scholar of the First Sin heap expander

Small DS2S 1.03 utility that allows for configuring heap sizes for modding purposes to prevent memory exhaustion and crashes.

It should be loaded early, before the game is able to initialize its heap allocators.

For legacy modengine compatibility, it can be loaded as a standalone dinput8 proxy (by renaming "ds2s_heap_x.dll" into "dinput8.dll") or chainloaded:

*modengine.ini*
```
[misc]
chainDInput8DLLPath="/path/to/ds2s_heap_x.dll"
```

"ds2s_heap_x.toml", the config file, contains multipliers for most of the game's permanent heap sizes. The heaps are only initialized once, so restarting the game is necessary after editing the config. If the config file is missing, it will be created with default values in the same directory as "ds2s_heap_x.dll".

WIP
