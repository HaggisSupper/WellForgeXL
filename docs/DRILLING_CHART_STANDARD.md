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

- SLB Oilfield Review, *Slide Drilling—Farther and Faster*: https://www.slb.com/-/media/files/oilfield-review/04-slide-drilling-english
- AADE, *A Graphical Hole Monitoring Procedure...*: https://www.aade.org/download_file/2821/492
- Texas A&M, stress-based torque-and-drag roadmap discussion: https://oaktrust.library.tamu.edu/bitstreams/3fab200c-435c-448a-a9de-51c902a02be3/download
- WELLPLAN ECD-versus-depth example: https://utpedia.utp.edu.my/3357/1/FYP_Dissertation_-_William.pdf
- OMV drilling roadmap and hydraulics overlay discussion: https://pure.unileoben.ac.at/ws/files/7787010/AC16358266.pdf
- Halliburton WELLPLAN product interface: https://www.halliburton.com/en/products/engineers-desktop-suite/wellplan-software
- Innova Engineering documentation: https://docs.innova-drilling.com/introduction/innova-engineering-manual/innova-engineering/1.0-software-overview
- AADE torque-and-drag output convention: https://www.aade.org/download_file/2710/491
