# gtfsfabrik

*Current Version*: 0.0.1

***THIS IS A WORK IN PROGRESS*** 
I am super excited about this project, but it is still (very much) a work in progress.
While gtfsfabrik is still in development you can expect:

- Incomplete or innacurate documentation
- Messy (at times spaghetti-ish) code
- Feature changes as implementation details might change certain design choices
- (Hopefully) A steady stream of commits and progress

## What is gtfsfabrik (or what will it be)?
gtfsfabrik is a command line tool (written in rust) I am creating for working with transit data in an 
intuitive, painless, and fast way, without giving up any capability or features. gtfsfabrik provides 
a structured workspace called a **fabrik** (from the German for **factory**) where you can model transit
systems using GTFS ([see GTFS info here](https://gtfs.org/getting-started/what-is-gtfs/)) and OpenStreetMap 
data allowing you to model various scenarios (such as removing or adding stations and lines,
changing vehicles on a line, and more) with modular, hot-swapable components to support creating mutliple different
scenarios. Think of it a bit like Git meets GIS and GTFS meets Mr. Potato Head.

## Planned Features
- Import and use existing GTFS data.
- Create any number of scenarios to analyze, e.g., compare a no-build vs. build scenario for a new transit line/station.
- Easily generate new GTFS data for your scenario, including recalculating stop times and schedules
- Create *spec-compliant* GTFS data that you can use wherever you need to use GTFS data (such as existing GIS software).
- Generate useful summaries for your scenarios, including destination/job accessibility metrics and isochrones
- Calculate travel times between points (which you specify) and calculate Origin-Destination cost matrices
- Use Census data for in-depth transit analyses *I am still considering how capable I can make this feature*
- Analyze construction costs of transit projects (new lines, etc)
- Calculate and analyze operational expenses for transit systems

## Design Goals
Below are some guiding philosophies in my development of gtfsfabrik

#### Easy to Use:
Anyone who understands how transit systems work, and the basics of GIS, can use gtfsfabrik to work with transit
data in an intuitive and easy way.

#### Worthwile:
gtfsfabrik should be able to replace just about any part of your workflow that deals with GTFS and transit data.
You should never have to open *(insert your GIS/GTFS tool of choice here)* to work with transit data.

#### Reproducable:
As opposed to current GTFS tools, you should be able to easily reproduce the results you get. Given the same
inputs (initial data and commands) you should always get the same output. Further, it should be easy
for any two people to share the steps they used to generate their data, whether it be GTFS data, analysis data, etc.

#### Automatable and Portable:
Reproducability and determinism mean that gtfsfabrik can support workload automation. You should be able to write
a simple script (such as a bash script or a gtfsfabrik python script) that runs desired commands so that 
a scenario or analysis can easily be run on any computer by anyone. i.e., gtfsfabrik should be easily portable
from one system to another.

#### Sensible (Easy opt-out) Defaults
gtfsfabrik should have reasonable/sensible defaults for anything that users might not be able to specify 
themselves, such as: 

- Vehicles (for calculating stop times/schedules)
- Construction costs (Should not estimate a subway costing $50 million/mile in NYC, or $2 billion/mile in Boise)

Should users want to provide their own information, however, gtfsfabrik should take 
full advantage of all data it can get, and use this information to create more accurate outputs.

#### Easy Configuration
Any configuration should be easily accessible through ***readable*** config files (using TOML) so that users 
can easily write their own config files, rather than dealing with the CLI. The CLI, however, should be
good enough so that users who do not wish to do so, do not need to.

## Development Notes

As mentioned previously, this is very much a work in progress, and if you're seeing this now, hopefully some of
the afore-mentioned features and design goals will have been implemented!

This is my first project that I am writing in Rust, and I am pretty new to the language, so my code may 
well be a bit messy or strange. If you have advice, questions, or suggestions for features please
feel free to email me at aaronbrumph@gmail.com.

## License

MIT
