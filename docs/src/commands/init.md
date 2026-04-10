# Init Command

--- 

The init command is the first step in using gtfsfabrik. Running 'gtfsfabrik init'
will create a new fabrik for you to work in! You **must** provide a path as an argument
when you use the init command, but all other arguments are optional. Supports
either relative paths or absolute paths.

## Usage
```shell
$ gtfsfabrik init [PATH] [OPTIONS]
```

## Arguments
| Argument | Description |
|---|---|
| `PATH` | The path where the fabrik will be created. If there are missing directories along the way, you will be asked whether you would like to create them. |


## Options
| Flag | Default | Description |
|---|---|---|
| `--gtfs` | None | One or more paths to GTFS data. Can be a zip file, unzipped folder, or a folder containing multiple GTFS feeds (as zips or unzipped folders). |
| `--osm` | None | Path to an OSM PBF file to initialize the fabrik with. |
| `--place` | None | Name of the base place to use as the fabriks's main location (e.g. `"Chicago, IL, USA"`). Can be used to fetch OSM/GTFS/Census data if you have not provided it. |
| `--geoscope` | None | Geographic scope for the fabrik. One of `place`, `county`, `msa`, or `csa`. Requires `--place`. |
| `--ridership` | None | Path to a ridership CSV file of the required format (see [Ridership info](../input-data/ridership.md)). |
| `--usegit` | `true` | Whether to initialize a git repository in the fabrik (with sensible defaults!). See [Using Git with gtfsfabrik](../tips/using-git.md) |

## Examples

### Blank Fabrik 
First, to get a sense of what setting up a fabrik with init looks like,
let's create a blank new fabrik called `chicago-fabrik`:
```shell
$ gtfsfabrik init chicago-fabrik
```
This will create a fabrik called `chicago-fabrik` in your current directory!
You can then cd into this directory, and if you run ls, you will see the following:
```shell
$ cd chicago-fabrik
$ ls
```
#### Result:
<!-- TODO: add ls results -->

Now you could also choose to provide an absolute path instead:
```shell
$ gtfsfabrik init /home/your-name/transit/fabriks/chicago-fabrik
```
You would now have a `chicago-fabrik` fabrik in `/home/your-name/transit/fabriks`.

Having a blank fabrik with nothing in it is not particularly exciting though. 
Let's look at a more complicated example...

### Chicago (City Limits)
Say you wanted to create a new fabrik to analyze the effect of removing a transit station on ridership.
For our example, we will set up a fabrik to do so for the Monroe Blue Line station
on the Chicago Transit Authority's 'L'
([See my blog if you would like to see more info about this example](https://aaronrumph.com/portfolio/should-the-cta-shed-some-wait/)).

When setting up your fabrik with init, you can provide your own GTFS, OpenStreetMap, or ridership data
so that you can jump straight into whatever you need to do! Let's look at an example for Chicago
in which we provide our own GTFS data for the CTA, our own OSM PBF file for the street network, and specify the place
and geographic scope that our fabrik will model:
```shell
$ gtfsfabrik init chicago-fabrik --gtfs ~/data/cta-gtfs.zip --osm ~/data/chicago-osm.pbf --place "Chicago, IL, USA" --geoscope place
```
Oof, quite a long command for sure! Don't worry if you would prefer not to have to run such a long command,
you can always add GTFS, OSM, and ridership data later! However, initializing a fabrik with these options
means that our fabrik is pretty much ready to go out of the box. With just that one command, you will already
be set up to:

- Change the provided GTFS data, removing/adding stations, lines, new vehicles, and more.
- Calculate travel times between points accessible via the transit system (including Origin Destination matrices).
- Create isochrones around points in the area

And much more!

## Next Steps:
Now that you've created a fabrik, you can start using gtfsfabrik to manipulate and analyze transit data!
Let's look at some commands that you can use to do this...
