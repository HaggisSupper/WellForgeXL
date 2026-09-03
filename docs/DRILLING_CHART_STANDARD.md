# WellForge drilling chart standard

## Depth-roadmap contract

Every depth-indexed engineering chart uses a true XY scatter chart:

- calculated response on the horizontal X axis;
- MD or TVD on the vertical Y axis;
- zero/minimum depth at the top and depth increasing downward;
- response-axis labels and title at the top;
- measured, modeled, and limit/threshold series overlaid when they share the same physical dimension;
- separate charts when the series use incompatible dimensions or scales.

This follows the topology of the supplied `T&D 4.002b.xlsm` reference and the common drilling-roadmap convention shown in the sources below. Hookload roadmaps overlay modeled and observed/current operating curves against MD; hydraulics roadmaps show pressure/ECD responses against depth; and operating limits are superimposed only when the units are comparable.

## Suite compositions

- Torque and drag: POOH/RIH hookload, operating torque, axial load with sinusoidal/helical buckling limits, and per-operation roadmaps.
- Hydraulics: pressure components vs MD, static density/ECD/ECD screen vs MD, velocity/minimum annular-velocity screen vs MD, and a separate nozzle-diameter pressure envelope.
- Directional: plan view, vertical section with reversed TVD, inclination/azimuth vs MD, plan/actual DLS vs MD, signed positional errors vs MD, and horizontal/3D error vs MD.
- BHA: frequency strip chart, bending-stress strip chart, component tendency heatmap, and PolarPlotter-style WOB/toolface rose plot.

## Integrated engineering-review compositions

- Torque and drag: one synchronized dashboard overlays PUW, SOW, BKR, SLD, ROT and DRLG axial responses with observed/mock hookload, tension rating, sinusoidal buckling and helical buckling. Separate synchronized roadmaps show operating torque/observed torque/torsional rating, inclination, and low/base/high friction sensitivity.
- Hydraulics: one synchronized dashboard shows low/base/high flow families for total pressure, ECD and annular velocity. Static/hydrostatic references and configured pressure, ECD and transport limits remain on the same physical scale as the governed response. Nozzle optimization remains a conventional numeric XY plot.
- Selected-depth readers use the nearest calculated station and display the associated modeled value, observed value, margin and governing state.
- `Chart Settings` persists selected MD, visibility controls, sensitivity multipliers, well-context visibility and report composition with the case.

Operation and semantic colors are stable across charts. Observed data uses a dark trace; limits use risk colors; sensitivity families progress from neutral grey through teal to green/red where appropriate. The packaged observed series is explicitly mock data and must not be presented as a field measurement.

## References

Public-domain / academic references only. Vendor, operator, and product names
are intentionally omitted from this document (see `docs/REFERENCE_ARCHIVE.md`,
usage rule 1).

- Industry technical-review article on slide drilling practice.
- Trade-association paper on graphical hole-condition monitoring procedure.
- University thesis on stress-based torque-and-drag roadmap presentation.
- University thesis presenting an ECD-versus-depth dashboard example.
- University thesis on drilling roadmap with hydraulics overlay.
- Trade-association paper on torque-and-drag output conventions.
