# BHA mechanics scope

This crate provides two bounded, deterministic capabilities:

- A catalog model for BHA components and ordered assemblies. Components are
  validated for finite, physically positive dimensions and material values; an
  assembly begins at the bit.
- A local, linear-elastic 2-D Euler–Bernoulli beam element with degrees of
  freedom `[y1, theta1, y2, theta2]`. It reports local end shear, end moment,
  bending stress, maximum combined normal stress, and strain energy for a
  supplied displacement state. The reported maximum normal stress is
  `|axial stress| + maximum bending stress`; it is not an equivalent-stress
  calculation.

The beam element is appropriate for small-deflection, prismatic-member checks
when a local planar representation and supplied displacements are suitable. It
does not assemble a system, solve boundary conditions or loads, model contact,
torsion, nonlinear material behavior, dynamics, or a spatial finite-element
model. Callers must select inputs and engineering acceptance criteria for their
specific operating conditions.

All public calculations reject non-finite or physically invalid inputs. They
also reject finite input combinations that overflow a reported matrix or
response value.
