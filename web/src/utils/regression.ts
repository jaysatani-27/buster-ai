import { createDayjsDate } from './date';

export const calculateLinearSlope = (data: number[]) => {
  const n = data.length;
  let sumX = 0;
  let sumY = 0;
  let sumXY = 0;
  let sumXX = 0;

  // Calculate sums needed for linear regression
  for (let i = 0; i < n; i++) {
    sumX += i;
    sumY += data[i];
    sumXY += i * data[i];
    sumXX += i * i;
  }

  // Calculate slope and intercept
  const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
  const intercept = (sumY - slope * sumX) / n;
  const slopeData = data.map((item, index) => slope * index + intercept);

  // Return coefficients and function to calculate y values
  return {
    slope,
    slopeData,
    intercept,
    equation: `y = ${slope.toFixed(1)}x + ${intercept.toFixed(1)}`
  };
};

export const calculateLinearSlopeByDate = (data: number[], dates: string[]) => {
  const n = data.length;

  // Convert dates to timestamps (milliseconds since epoch)
  const timestamps = dates.map((date) => createDayjsDate(date).valueOf());

  let sumX = 0;
  let sumY = 0;
  let sumXY = 0;
  let sumXX = 0;

  // Calculate sums needed for linear regression
  for (let i = 0; i < n; i++) {
    sumX += timestamps[i];
    sumY += data[i];
    sumXY += timestamps[i] * data[i];
    sumXX += timestamps[i] * timestamps[i];
  }

  // Calculate slope and intercept
  const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
  const intercept = (sumY - slope * sumX) / n;

  // Calculate predicted values for each date point
  const slopeData = timestamps.map((timestamp) => slope * timestamp + intercept);

  // Calculate milliseconds per unit change
  const msPerUnit = 1 / slope;

  // Convert to more readable "change per day" rate
  const changePerDay = slope * (24 * 60 * 60 * 1000); // milliseconds in a day

  return {
    slope,
    slopeData,
    intercept,
    changePerDay,
    equation: `y = ${slope.toExponential(2)}x + ${intercept.toFixed(1)}`,
    predict: (date: string) => {
      const timestamp = createDayjsDate(date).valueOf();
      return slope * timestamp + intercept;
    }
  };
};

export const calculateLogarithmicRegression = (data: { x: number; y: number }[]) => {
  const n = data.length;
  let sumX = 0;
  let sumY = 0;
  let sumLnX = 0;
  let sumYLnX = 0;
  let sumLnX2 = 0;

  // Calculate sums needed for logarithmic regression
  for (let i = 0; i < n; i++) {
    const { x, y } = data[i];
    const lnX = Math.log(x);

    sumX += x;
    sumY += y;
    sumLnX += lnX;
    sumYLnX += y * lnX;
    sumLnX2 += lnX * lnX;
  }

  // Calculate coefficients a and b
  const b = (n * sumYLnX - sumY * sumLnX) / (n * sumLnX2 - sumLnX * sumLnX);
  const a = (sumY - b * sumLnX) / n;

  const predict = (x: number) => {
    return a + b * Math.log(x);
  };

  const slopeData = data.map((item) => {
    return predict(item.x);
  });

  // Return coefficients and function to calculate y values
  return {
    a,
    b,
    slopeData,
    equation: `y = ${a.toFixed(1)} + ${b.toFixed(1)} * ln(x)`,
    predict
  };
};

export const calculateExponentialRegression = (data: { x: number; y: number }[]) => {
  const n = data.length;

  // Validate data - all y values must be positive
  if (data.some((point) => point.y <= 0)) {
    throw new Error('Exponential regression requires all y values to be positive');
  }

  let sumX = 0;
  let sumLnY = 0;
  let sumXLnY = 0;
  let sumXX = 0;
  let sumY = 0;

  // Calculate sums needed for exponential regression
  for (let i = 0; i < n; i++) {
    const { x, y } = data[i];
    const lnY = Math.log(y);

    sumX += x;
    sumLnY += lnY;
    sumXLnY += x * lnY;
    sumXX += x * x;
    sumY += y;
  }

  // Calculate coefficients
  const b = (n * sumXLnY - sumX * sumLnY) / (n * sumXX - sumX * sumX);
  const a = Math.exp((sumLnY - b * sumX) / n);

  const predict = (x: number) => {
    return a * Math.exp(b * x);
  };

  // Calculate R-squared
  const yMean = sumY / n;
  let ssRes = 0;
  let ssTot = 0;

  data.forEach((point) => {
    const yPred = predict(point.x);
    ssRes += Math.pow(point.y - yPred, 2);
    ssTot += Math.pow(point.y - yMean, 2);
  });

  const rSquared = 1 - ssRes / ssTot;

  const slopeData = data.map((item) => predict(item.x));

  return {
    a,
    b,
    equation: `y = ${a.toFixed(3)} * e^(${b.toFixed(3)}x)`,
    predict,
    slopeData,
    rSquared
  };
};

export const calculatePolynomialRegression = (
  data: { x: number; y: number }[],
  degree: number = 2
) => {
  const n = data.length;

  // Compute sums for matrix elements
  let sumX = 0;
  let sumY = 0;
  let sumXX = 0;
  let sumXXX = 0;
  let sumXXXX = 0;
  let sumXY = 0;
  let sumXXY = 0;

  for (let i = 0; i < n; i++) {
    const { x, y } = data[i];
    const xx = x * x;
    const xxx = xx * x;
    const xxxx = xx * xx;

    sumX += x;
    sumY += y;
    sumXX += xx;
    sumXXX += xxx;
    sumXXXX += xxxx;
    sumXY += x * y;
    sumXXY += xx * y;
  }

  let a: number, b: number, c: number;

  if (degree === 1) {
    // Linear regression: y = ax + b
    const denominator = n * sumXX - sumX * sumX;
    a = (n * sumXY - sumX * sumY) / denominator;
    b = (sumY * sumXX - sumX * sumXY) / denominator;
    c = 0;

    return {
      coefficients: [b, a],
      equation: `y = ${b.toFixed(3)} + ${a.toFixed(3)}x`,
      predict: (x: number) => a * x + b,
      type: 'linear'
    };
  } else {
    // Quadratic regression: y = ax² + bx + c
    const matrix = [
      [n, sumX, sumXX],
      [sumX, sumXX, sumXXX],
      [sumXX, sumXXX, sumXXXX]
    ];

    const vector = [sumY, sumXY, sumXXY];

    // Solve system of equations using Cramer's rule
    const D = determinant3x3(matrix);

    const matrix1 = [
      [vector[0], matrix[0][1], matrix[0][2]],
      [vector[1], matrix[1][1], matrix[1][2]],
      [vector[2], matrix[2][1], matrix[2][2]]
    ];

    const matrix2 = [
      [matrix[0][0], vector[0], matrix[0][2]],
      [matrix[1][0], vector[1], matrix[1][2]],
      [matrix[2][0], vector[2], matrix[2][2]]
    ];

    const matrix3 = [
      [matrix[0][0], matrix[0][1], vector[0]],
      [matrix[1][0], matrix[1][1], vector[1]],
      [matrix[2][0], matrix[2][1], vector[2]]
    ];

    c = determinant3x3(matrix1) / D;
    b = determinant3x3(matrix2) / D;
    a = determinant3x3(matrix3) / D;

    const predict = (x: number) => {
      return a * x * x + b * x + c;
    };

    const slopeData: number[] = data.map((item) => {
      return predict(item.x);
    });

    return {
      coefficients: [c, b, a],
      equation: `y = ${a.toFixed(3)}x² + ${b.toFixed(3)}x + ${c.toFixed(3)}`,
      predict,
      type: 'quadratic',
      slopeData: slopeData
    };
  }
};

// Helper function to calculate 3x3 determinant
const determinant3x3 = (matrix: number[][]) => {
  return (
    matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1]) -
    matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0]) +
    matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0])
  );
};

export const calculateLinearRegression = (data: { x: number; y: number }[]) => {
  const n = data.length;
  let sumX = 0;
  let sumY = 0;
  let sumXY = 0;
  let sumXX = 0;

  // Calculate sums needed for linear regression
  for (let i = 0; i < n; i++) {
    const { x, y } = data[i];
    sumX += x;
    sumY += y;
    sumXY += x * y;
    sumXX += x * x;
  }

  // Calculate slope (m) and intercept (b)
  const predict = (x: number) => {
    return slope * x + intercept;
  };
  const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
  const intercept = (sumY - slope * sumX) / n;

  // Return the equation of the line
  return {
    slopeData: data.map((item) => predict(item.x)),
    intercept,
    equation: `y = ${slope.toFixed(2)}x + ${intercept.toFixed(2)}`,
    predict
  };
};
