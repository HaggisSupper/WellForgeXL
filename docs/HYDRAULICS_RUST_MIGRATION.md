# Hydraulics Rust migration

## Boundary rules

- Rust owns pressure, flow-regime, nozzle and ECD calculations. VBA remains a bounded SI input adapter and value-only presentation client.
- Every dimensional contract field carries an SI suffix. Internal equations use metres, seconds, kilograms, kelvin and pascals; display-unit conversion stays outside the solver.
- Ported routines receive physics-based names. Historical product, vendor, book and class names are not used as Rust function identifiers.
- Contract `0.1.0` remains readable and selects the original screening behavior. It rejects every `0.2.0`-only field. Contract `0.2.0` requires explicit solver, nozzle-coefficient and surface-backpressure controls.
- Numerical parity is established with fixtures and limiting cases, not by reproducing control flow or defects from the source application.

## Delivered slice: pressure correlation and execution

Contract `0.2.0` adds:

- `HydraulicsSolverOptions.flow_correlation`
- `HydraulicsSolverOptions.compute_backend`
- independently fitted high-shear flow index for turbulent non-Newtonian response
- optional section TVD in metres
- nozzle discharge coefficient
- annulus surface backpressure in pascals
- flow-regime, TVD reference and complete circulating-pressure evidence

`darcy_weisbach_screening` retains the original WellForge Rust behavior for old requests, including its hard regime switch, and is not the preferred engineering correlation. `generalized_yield_power_law` evaluates Newtonian, Bingham, power-law and Herschel-Bulkley inputs through one SI-native wall-response formulation. Pipe and concentric-annulus geometry corrections are explicit, the turbulent response uses an independently fitted high-shear flow index, and the Fanning friction factor is blended continuously through transition.

`parallel_cpu` uses Rayon only for independent section evaluations. Collection and pressure summation return to request order, so serial and parallel results are deterministic and directly comparable. GPU execution is intentionally deferred: the present section count and branch-heavy scalar workload are too small to offset transfer, shader compilation and dispatch overhead.

The workbook therefore defaults to `darcy_weisbach_screening` and `serial_cpu`. `generalized_yield_power_law` and `parallel_cpu` remain selectable for controlled studies. Multicore execution is not treated as faster unless a release benchmark demonstrates at least a 20 percent p95 improvement and at least 1 ms absolute improvement at the applicable section count.

Nozzle optimization uses the CLI transport commands `validate-batch`, `run-batch` and `verify-batch`. Five candidates now require three bounded process launches instead of fifteen. A homogeneous batch prepares pipe/annulus section flow state once and applies each nozzle set in request order; a heterogeneous batch safely falls back to independent solves. Every result retains its own normalized request hash and payload hash.

Run `tools/Benchmark-WellForgeHydraulics.ps1` against a release executable to compare both workflows. The initial 20-sample Windows run measured 155.290 ms versus 32.077 ms median (4.84x) and a 78.4 percent p95 reduction. These figures are machine-specific evidence for batching, not permission to default every workload to multicore execution.

## Verification ladder

1. Dimensional limiting cases: the zero-yield, flow-index-one pipe response must recover conventional Newtonian Reynolds number.
2. Correlation invariants: every accepted rheology produces finite positive wall response and pressure loss.
3. Transition checks: friction remains finite and continuous on both sides of each regime boundary.
4. Execution parity: serial and parallel backends preserve record order and produce identical numerical results.
5. Pressure-budget checks: nozzle loss follows inverse-square discharge-coefficient scaling; ECD uses TVD and annulus backpressure only.
6. External parity: approved field-unit reference cases are converted once at the fixture boundary and compared in SI.

The frozen `0.1.0` request/result pair is captured under `wellforge-hydraulics-fixtures/data`. The current executable must reproduce the pre-migration JSON value and embedded request/result hashes exactly.

The generalized correlation remains an opt-in engineering model until the external parity matrix and licensed-standard clause review are accepted. Selecting it does not by itself claim full standard compliance.

## Workbook research evidence

The static catalog under `research/drilling-calculation-workbooks` provides
comparison evidence, not implementation authority. Its 41 hydraulics workbooks
contain 384,239 formula cells compacted into 1,059 structural families. Three
large Bingham/power-law/yield-power-law datasets contain 330,096 of those cells
but only 14 families, so they are parity datasets rather than independent model
counts. Workbook `2f31e7e9f41839d2` contributes the broadest operational
surface, including pressure loss, ECD, surge/swab, lag, well control, and survey
support. Each candidate still requires an independent standard or first-principles
acceptance case before migration.

No complete counter-current wellbore solver was identified in the locally
extracted formulas, VBA source, or p-code analysis. Workbook `dc888cd53f297b84`
contributes property conversions and a small cylindrical radial-resistance
network; workbook `348dd5eb51c7fac7` contributes a linear formation-temperature
profile and gradient fixtures. Workbook `b147eb91b2f2fd18` concerns elastomer
expansion, fit, wear, and life derating, not down-going string/up-going annulus
heat exchange. These sources can seed adversarial fixtures, but they do not
change the neutral model boundary below.

## Planned counter-current temperature layer

Temperature will be implemented as a separate `wellforge-wellbore-thermal` contract/core pair, coupled to hydraulics through section-local fluid states. Its public model name will be `counter_current_wellbore_exchange`.

The model will represent:

- down-going fluid in the string;
- up-going fluid in the annulus;
- heat transfer across the tubular wall;
- heat transfer between annulus and formation;
- formation temperature and transient radial thermal resistance;
- pressure/temperature-dependent density, heat capacity, conductivity and rheology.

The axial coordinate `s` increases with measured depth. Conventional circulation
uses positive mass flow in `kg/s` toward increasing `s` in the string and the
same magnitude toward decreasing `s` in the annulus. Physical reverse
circulation is a separate passage assignment and inlet-boundary configuration,
not a reversal of numerical marching order. Heat-transfer area is integrated
over MD, including horizontal cells, while formation temperature is evaluated
from TVD. The initial steady equations use linear conductances `G_sa` and `G_af`
in `W/(m*K)`:

```text
(m_dot * cp_s) dT_s/ds = G_sa (T_a - T_s)
-(m_dot * cp_a) dT_a/ds = G_sa (T_s - T_a) + G_af (T_f - T_a)
```

The boundary conditions are one surface string inlet, `T_s(0) = T_in`, and bottom enthalpy continuity across the turnaround. With no specified bit heat, `h_a(L) = h_s(L)`; a future bit heat source enters explicitly as `m_dot * (h_a(L) - h_s(L)) = q_bit`. The annulus surface outlet `T_a(0)` is solved, not supplied as a second inlet.

Hydraulics will hand the thermal lane an internal `HydraulicFieldSi`: ordered
MD/TVD cells, face pressure, signed passage mass flow in `kg/s`, density,
viscosity, and velocity. Any mass flux is a separate `kg/(m^2*s)` field. Thermal
output returns cell temperatures, properties, and heat rates without changing
the public hydraulics result contract. The reference thermal slice uses constant
properties, prescribed formation temperature, and prescribed linear
conductances. Transient radial formation state is a later contract addition.

The first implementation should use an implicit block-banded finite-volume solve with a reported energy residual. Hydraulics and temperature iterate until both pressure and temperature changes satisfy explicit tolerances; neither solver silently owns the other solver's convergence.

Thermal acceleration should begin with parallel scenario and sensitivity batches. A GPU backend is justified only after a benchmarked grid-size threshold and must pass the same conservation and deterministic-tolerance fixtures as the CPU reference path.

Required thermal acceptance cases are:

- zero string/annulus and formation conductance preserves the supplied string inlet through the turnaround and outlet;
- uniform inlet and formation temperature remains uniform everywhere;
- insulated formation conserves combined string/annulus enthalpy;
- increasing tubular conductance reduces the string/annulus temperature difference;
- reversing only the numerical solve order leaves the solution invariant;
- physical reverse circulation swaps passage assignment and inlet boundary while preserving energy conservation;
- grid refinement converges outlet temperatures and energy residual;
- horizontal cells exchange heat over MD while using TVD for formation temperature;
- a coupled pressure/temperature case converges to the isothermal result as all temperature coefficients approach zero.

## Next bounded slices

1. Add the external SI parity matrix across all four rheologies and both flow geometries.
2. Benchmark serial and multicore execution at `N = 1, 8, 64, 1024, 8192` sections before enabling acceleration by default.
3. Replace adapted workbook diameters with explicit mixed flow-path geometry records.
4. Add rotation-related annular loss and a complete pressure budget after independent validation.
5. Introduce the thermal contract and steady counter-current finite-volume reference solver.
6. Couple temperature-dependent fluid properties only after standalone thermal conservation tests pass.
