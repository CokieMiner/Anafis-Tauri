import csv, random, math

random.seed(42)
N = 30

# True parameters
a, b, c, d, e = 2.5, -1.3, 0.8, 3.1, -0.5

# Generate independent variables
x = [round(random.uniform(-5, 5), 3) for _ in range(N)]
y = [round(random.uniform(-5, 5), 3) for _ in range(N)]
z = [round(random.uniform(-5, 5), 3) for _ in range(N)]
w = [round(random.uniform(-5, 5), 3) for _ in range(N)]

# 1D: f = a*x + b  =>  2.5*x + (-1.3)
f1 = [round(a*x[i] + b + random.gauss(0, 0.3), 4) for i in range(N)]
# 2D: f = a*x + b*y + c  =>  2.5*x + (-1.3)*y + 0.8
f2 = [round(a*x[i] + b*y[i] + c + random.gauss(0, 0.4), 4) for i in range(N)]
# 3D: f = a*x + b*y + c*z + d  =>  2.5*x + (-1.3)*y + 0.8*z + 3.1
f3 = [round(a*x[i] + b*y[i] + c*z[i] + d + random.gauss(0, 0.5), 4) for i in range(N)]
# 4D: f = a*x + b*y + c*z + d*w + e
f4 = [round(a*x[i] + b*y[i] + c*z[i] + d*w[i] + e + random.gauss(0, 0.6), 4) for i in range(N)]

# Uncertainties
sf = [round(random.uniform(0.2, 0.8), 3) for _ in range(N)]

with open('/home/cokieminer/Documentos/AnaFis-Tauri/test_nd_fitting.csv', 'w', newline='') as f:
    writer = csv.writer(f)
    writer.writerow(['x', 'y', 'z', 'w', 'f1', 'f2', 'f3', 'f4', 'sf'])
    for i in range(N):
        writer.writerow([x[i], y[i], z[i], w[i], f1[i], f2[i], f3[i], f4[i], sf[i]])

print('CSV written with 30 data points')
print()
print('Test formulas (true params: a=2.5, b=-1.3, c=0.8, d=3.1, e=-0.5):')
print('1D: a*x + b           | vars: x      | params: a, b')
print('2D: a*x + b*y + c     | vars: x, y   | params: a, b, c')
print('3D: a*x + b*y + c*z + d   | vars: x, y, z | params: a, b, c, d')
print('4D: a*x + b*y + c*z + d*w + e | vars: x, y, z, w | params: a, b, c, d, e')