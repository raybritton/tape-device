## Piped Device

External programs may want to host an instance of Tape Device to interact with it, for example a debugger.

When executing a tape file, if the tape device is run with `--piped`, e.g. `./tape_device example.tape --piped` then the program will not run automatically and instead will rely on sending and receiving commands over stdin and stdout.

Output and input for the tape program will also be sent over stdin and stdout via the host program.

### Usage

Input commands should be sent to the device, note that the order matters: the device will process commands until step is received at which point it will execute the next command.
This means if nothing is sent to the device it will do anything. If step is sent twice, both will be execute before anything else.

### Commands

Commands are sent/received in the format `<prefix><content>` as bytes

#### Input

| Name | Prefix | Content | Example | Bytes | Notes |
|----------|----------|-------|-----|-----|----|
| Step | e | - | `e` | `x65` | Executes the next instruction, unless there's a breakpoint |
| Step, ignoring breakpoints | f | - | `f` | `x66` | Executes the next instruction, ignoring breakpoints |
| Set breakpoint | b | address (2 bytes) | `s,451` | `x7301C3` | Sets a breakpoint |
| Clear breakpoint | c | address (2 bytes) | `c,1` | `x990001` | Clears a breakpoint |
| Request Dump | d | - | `d` | `x64` | Tells device to send JSON string of registers, etc |
| Input Key | i | char (1 byte) | `i,T` | `x6954` | Send one key press to device | 
| Input String | t | String | `t` | `x6954` | Send one key press to device | 
| Request Memory | m | 2x addr (2 bytes each) | `t` | `x6954` | Send one key press to device | 
| Request Stack | s | - | `t` | `x6954` | Send one key press to device | 
| Set Memory | n | addr, len, bytes | `n,450,2,4,5` | `x6E01C2..` | Send one key press to device | 
| Set Register | r | id, value | `r,1,50` | `x720132` | Send one key press to device |  
| Stop | f | - | `f` | `x66` | Tell Piped Device to finish |  

#### Output

| Name | Prefix | Content | Example | Bytes | Notes |
|----------|----------|-------|-----|-----|----|
| Output | o | String | `o,4,Test` | `x6F045465..` | Standard output from the tape program |
| Error Output | e | String | `e,5,Crash` | `x65054372..` | Error output from the tape device |
| Breakpoint hit | h | address (2 bytes) | `h,5` | `x680005` | Sent when 'Step' is sent but there's a breakpoint |
| Dump output | d | String | `d,200,{"p..` | `xC87B22..` | JSON string of registers, etc |
| Memory output | m | len,bytes | `m,200,0,0..` | `xC87B22..` | Output of requested memory range |
| Stack output | s | len,bytes | `s,8,3,10..` | `xC87B22..` | Output of requested stack memory |
| Key Requested | k | - | `k` | `x6B` | Tape program is waiting for key press |
| String Requested | t | - | `t` | `x74` | Tape program is waiting for a string |
| End of program | f | - | `f` | `x66` | Tape program has finished (EoF or HALT) |
| Crashed | c | - | `c` | `x63` | Tape program has crashed |

#### Notes

* Strings are max 255 bytes long, the device will send multiple commands if there's more than 255 chars. A string param must be prefixed with it's length (one byte).

#### Dump structure

```json
{
  "pc": 0,
  "acc": 0,
  "sp": 0,
  "fp": 0,
  "data_reg": [
    0,
    0,
    0,
    0
  ],
  "addr_reg": [
    0,
    0
  ],
  "overflowed": false
}
```

#### Supported Keys for 'Input Key'

 * `a-z`
 * `A-Z`
 * `0-9`
 * `!@£$%^&*()_+-={}:"|<>?,./;'\\[]`~`
 * `<escape>`
 * `<space>`
 * `<return>`
 * `<tab>`
 * `<backspace>`