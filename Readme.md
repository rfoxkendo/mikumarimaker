## Products of this repository:

Two binaries here:
*  mikumarimaker takes a raw mikumari time data file and makes a ring item file.
*  defenestrator takes the output of e.g. mikumarimaker and output defenestrated ring items.

###  mikumarimaker

This program takes raw mikumari High precision TDC frame files and produces ring item frame files.
These have the following characteristics:

*  The data prior to the first heartbeat are discarded.
*  The item type is 51  - time frames.
*  The timestamp in the body header is the computed timestamp of the heartbeat that starts the frame.  By computed timestamp I mean that the first frame number is assigned a timestamp of 0. Subsequent frames have a timestamp that is the frame number * the number of tdc ticks per frame.  This assumes that (as documented):
    - frames are 524.288&mu;seconds apart.
    - The TDC tick (LSB resolution) is 0.9765625pico seconds.

Note: the frame number relative to the start of data are internally maintained as a uint64_t.  Note the 64 bit timestamp will rollover after over 200 days.

The ring item body contains the following:

|  Contents      | Size    |  Notes | 
|----------------|---------|--------|
| Raw frame number | uint64_t | only the least significant bits are meaningful. |
| Raw hit        | uint16_t   | The raw TDC value (rising or falling edges). |
|    ...         |   ...      | ...|

Where there will be as many raw hit values as there are up to the next heartbeat.
The following are filtered out:
* Delimeter 1 since the information it has is implicit in the ring item size.
* Throttle words.  Note that these could easily be added back if desired.

Usage of the program:

```
mikumarimaker infile outfile
```

* infile is the path to the file that contains raw mikumari data.
* outfile is the path to a file that will contain the resulting ring item frames.

Note that future work might allow outfile to be a ringbuffer e.g.  ```file://outfile``` or
```tcp:/localhost/ringname```

### defenestrator

Defenestration means to throw someone out a window.  In the context of FRIB/NSCLDAQ, it means to take windowed (frame files) and turn them into something 'else'.  For time data,
the framing is sort of maintained, but the time data are transformed into a format that
is no longer mikumari dependent.

The program can accept data from an input file, stdin, or ringbuffer and write data to an output file or stdout.  Future work may allow this to send output to an online ringbuffer.

The output of this program are a sequence of ring items.

*  The type of the ring items is ```PHYSICS_EVENT```
*  The ring items have body headers with timestamps that are the same as the timestamps in the input ring items.


Body contents are:

|     Contents     |   Size    | Notes   |
|------------------|-----------|---------|
| Absolute frame number | uint64_t | Only the bottom 16 bits are nonzero |
| Absolute hit     | uint16_t, uint64_t | See below |
|   ...            | ...                | ... |


Absolute hits  are a 16 bit word followed by a 64 bit word:

|  Contents       | Size     | Notes     |
|-----------------|----------|-----------|
| channel and edge| uint16_t | The top bit is set for falling edge, the remainder of the word is the channel number.
| Absolute time   | uint64_t | Time of the hit relative to the start of the run. |

Note the absolute time is computed fromt he mikumari hit time and the timestamp of the ring item.  It will roll over after over 200 days and the LSB as for the ring item timestamp is 0.9765625pico seconds.


Usage:
```
defenestrator source-uri output-file
```

Where:
|  parameter | Meaning                    |
|------------|----------------------------|
| source-uri | Is a the data source URI as per standard FRIB/NSCLDAQ URI format |
| output-file | is the path to the output ring item file ```-``` means stdout |

Note that as with standard FRIB/NSCLDAQ data sources, _source-uri_ can be ```-```
to indicate data should be taken from stdin.

Future developments will allow output-file to be an output-uri to allow data to go to an
online ringbuffer.