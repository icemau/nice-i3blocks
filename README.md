# Nice i3blocks

This is small collection of i3blocks that I use.

## Example

The following is an example `i3blocks.conf` file.
```toml
separator=true
separator_block_width=15

[cpu_usage]
command=cpu_usage
markup=pango
interval=persist
REFRESH_TIME=1

[memory]
command=mem_usage
markup=pango
interval=persist

[bandwidth]
command=bandwidth
interval=persist
markup=pango

[pomodoro]
command=pomodoro
interval=persist
format=json
markup=pango

[time]
command=date '+%d-%m-%Y %H:%M:%S'
interval=1
```
