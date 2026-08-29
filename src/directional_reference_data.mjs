export const directionalReferenceData = {
  metadata: [
    { key: 'wellName', value: 'MOCK HAWK RIDGE 7H' },
    { key: 'fieldPad', value: 'DEMO PAD WEST' },
    { key: 'rig', value: 'RIG SIM-7' },
    { key: 'surfaceNorthFt', value: 0 },
    { key: 'surfaceEastFt', value: 0 },
    { key: 'groundElevationFt', value: 5000 },
    { key: 'datum', value: 'RKB' },
    { key: 'northReference', value: 'Grid North' },
    { key: 'verticalSectionAzimuthDeg', value: 55 },
  ],
  inputs: [
    { key: 'planLengthUnit', value: 'ft' },
    { key: 'planAngleUnit', value: 'deg' },
    { key: 'surveyLengthUnit', value: 'ft' },
    { key: 'surveyAngleUnit', value: 'deg' },
    { key: 'fieldVerifiedMotorYieldDegPer100Ft', value: 8 },
    { key: 'rotaryBuildTendencyDegPer100Ft', value: 0.3 },
    { key: 'slideCalibrationWindowStands', value: 3 },
  ],
  plan: [
    [0, 0, 0, 20], [1, 500, 0, 20], [2, 1000, 0, 20], [3, 1500, 0, 20], [4, 2000, 0, 20], [5, 2500, 0, 20], [6, 3000, 0, 20], [7, 3500, 0, 20], [8, 4000, 0, 20], [9, 4500, 0, 20], [10, 5000, 0, 20], [11, 5125, 10, 22.222], [12, 5250, 20, 24.444], [13, 5375, 30, 26.667], [14, 5500, 40, 28.889], [15, 5625, 50, 31.111], [16, 5750, 60, 33.333], [17, 5875, 70, 35.556], [18, 6000, 80, 37.778], [19, 6125, 90, 40], [20, 6375, 90.021, 41.657], [21, 6625, 90.072, 43.295], [22, 6875, 90.173, 44.895], [23, 7125, 90.324, 46.439], [24, 7375, 90.505, 47.911], [25, 7625, 90.681, 49.295], [26, 7875, 90.81, 50.58], [27, 8125, 90.856, 51.755], [28, 8375, 90.797, 52.813], [29, 8625, 90.636, 53.75], [30, 8875, 90.398, 54.563], [31, 9125, 90.124, 55.255], [32, 9375, 89.861, 55.83], [33, 9625, 89.651, 56.295], [34, 9875, 89.52, 56.661], [35, 10125, 89.471, 56.939], [36, 10375, 89.486, 57.145], [37, 10625, 89.534, 57.295], [38, 10875, 89.581, 57.407], [39, 11125, 89.6, 57.5], [40, 11375, 89.581, 57.593], [41, 11625, 89.534, 57.705], [42, 11875, 89.486, 57.855], [43, 12125, 89.471, 58.061], [44, 12375, 89.52, 58.339], [45, 12625, 89.651, 58.705], [46, 12875, 89.861, 59.17], [47, 13125, 90.124, 59.745], [48, 13375, 90.398, 60.437], [49, 13625, 90.636, 61.25], [50, 13875, 90.797, 62.187], [51, 14125, 90.856, 63.245], [52, 14375, 90.81, 64.42], [53, 14625, 90.681, 65.705], [54, 14875, 90.505, 67.089], [55, 15125, 90.324, 68.561], [56, 15375, 90.173, 70.105], [57, 15625, 90.072, 71.705], [58, 15875, 90.021, 73.343], [59, 16125, 90, 75],
  ].map(([station, mdFt, incDeg, aziDeg]) => ({ station, mdFt, incDeg, aziDeg })),
  survey: [
    [0, 0, 0, 20], [1, 508.887, 0.487, 20.782], [2, 1010.415, 0.599, 21.186], [3, 1509.917, 0.445, 21.017], [4, 2007.759, 0.238, 20.356], [5, 2505.153, 0.524, 19.523], [6, 3003.539, 0.591, 18.921], [7, 3503.852, 0.397, 18.84], [8, 4006.045, 0.295, 19.319], [9, 4509.112, 0.554, 20.128], [10, 5011.602, 0.575, 20.875], [11, 5137.358, 11.802, 24.711], [12, 5261.108, 22.09, 27.432], [13, 5383.599, 31.569, 29.414], [14, 5506.224, 40.593, 30.777], [15, 5630.304, 49.826, 31.947], [16, 5756.421, 59.789, 33.447], [17, 5884.117, 70.508, 35.635], [18, 6012.131, 81.493, 38.528], [19, 6139.038, 92.075, 41.793], [20, 6388.979, 89.387, 44.424], [21, 6637.096, 89.502, 46.803], [22, 6884.461, 89.551, 49.094], [23, 7132.528, 89.548, 51.257], [24, 7382.396, 89.515, 53.258], [25, 7634.243, 89.468, 55.07], [26, 7887.24, 89.414, 56.674], [27, 8139.973, 89.35, 58.057], [28, 8391.159, 89.263, 59.219], [29, 8640.308, 89.141, 60.164], [30, 8887.964, 88.978, 60.908], [31, 9135.435, 88.779, 61.473], [32, 9384.121, 88.564, 61.887], [33, 9634.805, 88.366, 62.182], [34, 9887.249, 88.223, 62.395], [35, 10140.321, 88.172, 62.563], [36, 10392.567, 88.234, 62.724], [37, 10642.954, 88.41, 62.913], [38, 10891.397, 88.679, 63.162], [39, 11138.802, 89, 63.5], [40, 11386.604, 89.321, 63.948], [41, 11636.034, 89.59, 64.523], [42, 11887.496, 89.766, 65.234], [43, 12140.356, 89.828, 66.086], [44, 12393.269, 89.777, 67.074], [45, 12644.863, 89.634, 68.192], [46, 12894.436, 89.436, 69.427], [47, 13142.32, 89.221, 70.763], [48, 13389.704, 89.022, 72.182], [49, 13638.034, 88.859, 73.664], [50, 13888.273, 88.737, 75.192], [51, 14140.412, 88.65, 76.747], [52, 14393.472, 88.586, 78.314], [53, 14646.006, 88.532, 79.88], [54, 14896.835, 88.485, 81.437], [55, 15145.648, 88.452, 82.979], [56, 15393.163, 88.449, 84.504], [57, 15640.757, 88.498, 86.013], [58, 15889.77, 88.613, 87.51], [59, 16140.816, 88.8, 89],
  ].map(([station, mdFt, incDeg, aziDeg]) => ({ station, mdFt, incDeg, aziDeg })),
  targets: [
    { id: 'T1 - Landing / Heel Target', centerTvdFt: 5716.226822119378, centerNorthFt: 599.9517688193293, centerEastFt: 386.6362472673295, radiusFt: 35, entryIncDeg: 90, entryAziDeg: 40, note: 'Landing target tied to planned station 19' },
    { id: 'T2 - Mid-Lateral Turn Target', centerTvdFt: 5706.186399741192, centerNorthFt: 3669.7574999966523, centerEastFt: 4306.196772472447, radiusFt: 50, entryIncDeg: 89.6, entryAziDeg: 57.5, note: 'Mid-lateral target highlights planned azimuth turn' },
    { id: 'T3 - Planned TD Target', centerTvdFt: 5696.145977363006, centerNorthFt: 5924.729763759714, centerEastFt: 8744.86346727784, radiusFt: 75, entryIncDeg: 90, entryAziDeg: 75, note: 'TD target tied to planned station 59' },
  ],
  slideIntervals: [
    { stand: 1, dateSerial: 46255, mdInFt: 5000, mdOutFt: 5090, slideFt: 88, rotateFt: 2, commandedToolfaceDeg: 0 },
    { stand: 2, dateSerial: 46256, mdInFt: 5090, mdOutFt: 5180, slideFt: 87, rotateFt: 3, commandedToolfaceDeg: 0 },
    { stand: 3, dateSerial: 46257, mdInFt: 5180, mdOutFt: 5270, slideFt: 86, rotateFt: 4, commandedToolfaceDeg: 0 },
    { stand: 4, dateSerial: 46258, mdInFt: 5270, mdOutFt: 5360, slideFt: 85, rotateFt: 5, commandedToolfaceDeg: 0 },
    { stand: 5, dateSerial: 46259, mdInFt: 5360, mdOutFt: 5450, slideFt: 84, rotateFt: 6, commandedToolfaceDeg: 0 },
    { stand: 6, dateSerial: 46260, mdInFt: 5450, mdOutFt: 5540, slideFt: 83, rotateFt: 7, commandedToolfaceDeg: 0 },
  ],
  formationTops: [
    { id: 'formation-niobrara-a', name: 'Top Niobrara A', prognosedMdFt: 4600, prognosedTvdFt: 4550, localDipDeg: 1 },
    { id: 'formation-niobrara-b', name: 'Top Niobrara B', prognosedMdFt: 5100, prognosedTvdFt: 5040, localDipDeg: 1 },
    { id: 'formation-niobrara-c', name: 'Top Niobrara C', prognosedMdFt: 5350, prognosedTvdFt: 5280, localDipDeg: 1 },
    { id: 'formation-codell-landing-zone', name: 'Top Codell / Landing Zone', prognosedMdFt: 5600, prognosedTvdFt: 5716, localDipDeg: 1 },
  ],
};
