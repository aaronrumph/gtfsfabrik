# TODOs
--- 
## General
- [ ] Add documentation
- [ ] Clean out TODOS and BUGS

## RAPTOR
### Optimisation: 
- [ ] Multithread single query per Denning paper strat
- [ ] Binary search for trips rather than linear
- [ ] GPU whenever possible!
- [ ] Optimal caching of timetable, feed, id_maps
- [ ] Add name look up table for trips/stops for HRO
- [ ] Switch Vec<T> to Arc<T> for timetable?
- [ ] Bench: HashMaps vs Vec vs Arc

### Loader:
- [x] Finish Loader Implementation

### Transfers:
- [x] Simple straight line transfer implementation
- [ ] OSM walking routing transfer creation 

### Basic:
- [ ] Filter for day of!

## OSM
### PBF:
- [ ] Parse PBF into graph (petgraph?)
- [ ] Dijkstra's? A*?

## Commands
### Init:
- [ ] Add TOML files and finish init including moving gtfs data to data dir
- [ ] Check for all necessary columns in required files for GTFS
- [ ] Better PBF validation?

## Tests
### RAPTOR:
- [ ] Test that gives within 5% of r5py travel time
- [ ] Test logical certainties (arrive !< depart, self -> self = 0, etc)

