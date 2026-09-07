# Use checked-in NASA lunar ephemeris data

Apollo 18 will derive date-dependent lunar illumination and apparent Earth-centered orientation from NASA Scientific Visualization Studio's original annual Moon Phase and Libration JSON, committed with provenance, rather than calculating ephemerides in project code or depending on a live service. This keeps native and static WebAssembly rendering aligned, inspectable, and reproducible while accepting hourly source precision, larger assets, annual maintenance, bounded date coverage, and clear failure when a complete synodic month is unavailable.
