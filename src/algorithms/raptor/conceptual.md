# RAPTOR Conceptual Guide

Tyhis is basically just here for me to jot out in plain language
what RAPTOR procedure looks like

1. Load GTFS
2. Query Params
    a. Find closest start and end stops
    b. Set depart time/date
3. Initilize
  For Every Stop:
    a. Set best arrival time to infinity
    b. Set best "time this round" to infinity (global scope)
  Set arrival time at start to depart time
  Mark start stop as improved this round
3. Start Rounds:
  For Each Round (one additional transfer):
    a. Form queue: for each marked stop, add routes to queue
    b. Prune queue: only keep earliest earliest stop on each route
    c. Scan stops: Go to each stop and mark arrival time if better
    d. Check for better trip: While scanning stops, if find earlier trip on same route, switch
    d. Prune stops/trips: Ignore arrival times later than EDT at destination
    e. Check transfers: if can reach transferable stop sooner, mark and update
    e. Update: update earliest arrival times for stops where necessary
    f. Finish: Check if no stops marked
 
