#!/usr/bin/env python3

import tkinter as tk
import math
import numpy as np
import numpy.linalg as la
from scipy.optimize import brentq

WIDTH = 600
HEIGHT = 400

LEVEL_NAME = {
    1: "line",
    2: "quadratic bézier",
    3: "cubic bézier",
}

HELP_TEXT = "[+/-] change curve level  " \
            "[LMB] move point  " \
            "[RMB] move view  " \
            "[b] show bounding box"


def ipairs(it):
    """Make new iterator which returns rolling window of pairs from `it`."""
    it = iter(it)
    a = next(it)
    while True:
        b = next(it)
        yield a, b
        a = b


def solve_quadratic(a, b, c):
    """Find real roots of quadratic equation:

    a.x^2 + b.x + c = 0

    """
    return [r.real for r in np.poly1d([a, b, c]).r if r.imag == 0]


def solve_cubic(a, b, c, d):
    """Find real roots of cubic equation:

    a.x^3 + b.x^2 + c.x + d = 0

    """
    return [r.real for r in np.poly1d([a, b, c, d]).r if r.imag == 0]


def quadratic_bezier(t, p0, p1, p2):
    p0, p1, p2 = map(np.array, [p0, p1, p2])
    tc = 1 - t
    return tc*tc*p0 + 2*tc*t*p1 + t*t*p2


def cubic_bezier(t, p0, p1, p2, p3):
    p0, p1, p2, p3 = map(np.array, [p0, p1, p2, p3])
    tc = 1 - t
    return tc*tc*tc*p0 + 3*tc*tc*t*p1 + 3*tc*t*t*p2 + t*t*t*p3


def quadratic_derivative(t, p0, p1, p2):
    p0, p1, p2 = map(np.array, [p0, p1, p2])
    tc = 1 - t
    return 2*tc*(p1 - p0) + 2*t*(p2 - p1)


def cubic_derivative(t, p0, p1, p2, p3):
    p0, p1, p2, p3 = map(np.array, [p0, p1, p2, p3])
    tc = 1 - t
    return 3*tc*tc*(p1 - p0) + 6*tc*t*(p2 - p1) + 3*t*t*(p3 - p2)


def line_distance(p, p0, p1):
    p, p0, p1 = map(np.array, [p, p0, p1])
    m = p - p0
    a = p1 - p0
    t = m.dot(a) / a.dot(a)
    t = min(max(t, 0), 1)
    x = p0 + t * a
    # Determinant of this 2x2 matrix gives us orientation
    # of the two vectors. When clockwise rotation from p-p0 to p1-p0
    # takes less than 180°, the sign is positive, otherwise negative.
    # Note that in 3D, this is how we compute Z component of cross product
    # of two vectors.
    side = la.det([m, a])
    dist = math.copysign(la.norm(x - p), side)
    return dist, x, t


def quadratic_distance(p, p0, p1, p2):
    p, p0, p1, p2 = map(np.array, [p, p0, p1, p2])
    m = p0 - p
    a = p1 - p0
    b = p2 - p1 - a
    roots = solve_cubic(b.dot(b),
                        3*a.dot(b),
                        2*a.dot(a) + m.dot(b),
                        m.dot(a))
    # Find nearest point
    dist_min = None
    x_point = None
    x_t = None
    for t, x in [(0.0, p0), (1.0, p2)]:
        dist = la.norm(x - p)
        if dist_min is None or dist < dist_min:
            dist_min = dist
            x_point = x
            x_t = t
    for t in roots:
        if t < 0.0 or t > 1.0:
            continue
        x = quadratic_bezier(t, p0, p1, p2)
        dist = la.norm(x - p)
        if dist_min is None or dist < dist_min:
            dist_min = dist
            x_point = x
            x_t = t
    # Determine sign
    direction = quadratic_derivative(x_t, p0, p1, p2)
    side = la.det([p - x_point, direction])
    dist = math.copysign(dist_min, side)
    return dist, x_point, x_t


def cubic_distance(p, p0, p1, p2, p3):
    """Find distance from a point to cubic bézier curve.

    Input is the query point (P) and points defining the curve (P0, P1, ...).
    Output is tuple with the distance, nearest point on the curve (X)
    and function parameter leading to this point (t).

    We're looking for the roots of equation `PX.B'(t)=0`, where `PX = B(t) - P`.
    PX is a vector pointing towards query point (normal vector)
    B'(t) is derivative function, which gives tangent vector at point X.
    Dot product of these two vectors must be zero (vectors are perpendicular).

    """
    p, p0, p1, p2, p3 = map(np.array, [p, p0, p1, p2, p3])
    def f(t):
        return (cubic_bezier(t, p0, p1, p2, p3) - p).dot(cubic_derivative(t, p0, p1, p2, p3))
    # Partition t to `steps` intervals and solve the equation for each interval
    steps = 15
    bounds = [t / steps for t in range(0, steps + 1)]
    candidate_t = []
    for a, b in ipairs(bounds):
        try:
            t = brentq(f, a, b)
            candidate_t.append(t)
        except ValueError:
            # f(a) and f(b) must have different signs
            pass
    candidate_t += [0.0, 1.0]
    # Find nearest point in the candidates
    dist_min = None
    x_point, x_t = None, None
    for t in candidate_t:
        x = cubic_bezier(t, p0, p1, p2, p3)
        dist = la.norm(x - p)
        if dist_min is None or dist < dist_min:
            dist_min = dist
            x_point = x
            x_t = t
    # Determine sign
    direction = cubic_derivative(x_t, p0, p1, p2, p3)
    side = la.det([p - x_point, direction])
    dist = math.copysign(dist_min, side)
    return dist, x_point, x_t


def line_bbox(p0, p1):
    a = (min(p0[0], p1[0]), min(p0[1], p1[1]))
    b = (max(p0[0], p1[0]), max(p0[1], p1[1]))
    return a, b


def quadratic_bbox(p0, p1, p2):
    extrema = [p0, p2]
    # Solve the derivative B'(t) = 0, for each coordinate x, y.
    # This gives us t_x, t_y. If these are in curve interval (0..1)
    # then we get at most two extrema points of the parabola.
    candidates = []
    dx = p0[0] - 2*p1[0] + p2[0]
    if dx != 0:
        candidates.append((p0[0] - p1[0]) / dx)
    dy = p0[1] - 2*p1[1] + p2[1]
    if dy != 0:
        candidates.append((p0[1] - p1[1]) / dy)
    for t in candidates:
        if 0 <= t <= 1:
            p = quadratic_bezier(t, p0, p1, p2)
            extrema.append(p)
    xs = [p[0] for p in extrema]
    ys = [p[1] for p in extrema]
    a = (min(xs), min(ys))
    b = (max(xs), max(ys))
    return a, b


def cubic_bbox(p0, p1, p2, p3):
    p0, p1, p2, p3 = map(np.array, [p0, p1, p2, p3])
    extrema = [p0, p3]
    a = p3 - 3*p2 + 3*p1 - p0
    b = 2*(p2 - 2*p1 + p0)
    c = p1 - p0
    candidates = solve_quadratic(a[0], b[0], c[0])
    candidates += solve_quadratic(a[1], b[1], c[1])
    for t in candidates:
        if 0 <= t <= 1:
            p = cubic_bezier(t, p0, p1, p2, p3)
            extrema.append(p)
    xs = [p[0] for p in extrema]
    ys = [p[1] for p in extrema]
    a = (min(xs), min(ys))
    b = (max(xs), max(ys))
    return a, b


class App:

    def __init__(self):
        root = tk.Tk()
        self.label_help = tk.Label(root, text=HELP_TEXT, justify=tk.LEFT)
        self.label_help.pack()
        self.canvas = tk.Canvas(root, width=WIDTH, height=HEIGHT, bg="#555")
        self.canvas.pack(fill=tk.BOTH, expand=True)
        self.label_dist = tk.Label(root)
        self.label_dist.pack()
        self.canvas.bind("<ButtonPress-1>", self.on_left_press)
        self.canvas.bind("<ButtonRelease-1>", self.on_left_release)
        self.canvas.bind("<ButtonPress-3>", self.on_right_press)
        self.canvas.bind("<ButtonRelease-3>", self.on_right_release)
        self.canvas.bind("<Motion>", self.on_move)
        root.bind("<KeyPress-KP_Add>", self.on_key_plus)
        root.bind("<KeyPress-plus>", self.on_key_plus)
        root.bind("<KeyPress-KP_Subtract>", self.on_key_minus)
        root.bind("<KeyPress-minus>", self.on_key_minus)
        root.bind("<KeyPress-b>", self.on_key_bbox)
        root.bind("<KeyPress-d>", self.on_key_dump)
        root.wm_title(string="Segment distance")

        self.level = 3
        self.points = [(WIDTH * 0.6, HEIGHT * 0.2),  # X point
                       (WIDTH * 0.2, HEIGHT * 0.4),  # P0
                       (WIDTH * 0.5, HEIGHT * 0.8),  # P1
                       (WIDTH * 0.8, HEIGHT * 0.4),  # P2
                       (WIDTH * 0.8, HEIGHT * 0.8)]  # P3
        self.result = (None, None, None)
        self.show_bbox = False
        self.recreate_points()
        self.refresh()

        self.moving = None
        self.scrolling = False

    def main(self):
        tk.mainloop()

    def on_left_press(self, event):
        mx, my = self.canvas.canvasx(event.x), self.canvas.canvasy(event.y)
        tolerance = 10
        items = self.canvas.find_closest(mx, my)
        for item in items:
            if "point" in self.canvas.gettags(item):
                x1, y1, x2, y2 = self.canvas.coords(item)
                if (x1 - tolerance <= mx <= x2 + tolerance
                and y1 - tolerance <= my <= y2 + tolerance):
                    self.moving = item
                    return

    def on_left_release(self, _event):
        self.moving = None

    def on_right_press(self, event):
        self.scrolling = True
        self.canvas.scan_mark(event.x, event.y)

    def on_right_release(self, _event):
        self.scrolling = False

    def on_move(self, event):
        if self.moving is not None:
            item = self.moving
            idx = self.canvas.find_withtag("point").index(item)
            x, y = self.points[idx]
            rx = self.canvas.canvasx(event.x) - x
            ry = self.canvas.canvasy(event.y) - y
            self.canvas.move(item, rx, ry)
            self.points[idx] = (x + rx, y + ry)
            self.refresh()
        if self.scrolling:
            self.canvas.scan_dragto(event.x, event.y, 1)

    def on_key_plus(self, _event):
        if self.level < 3:
            self.level += 1
            self.recreate_points()
            self.refresh()

    def on_key_minus(self, _event):
        if self.level > 1:
            self.level -= 1
            self.recreate_points()
            self.refresh()

    def on_key_bbox(self, _event):
        self.show_bbox = not self.show_bbox
        self.refresh()

    def on_key_dump(self, _event):
        """Dump curve parameters to console"""
        print("Curve level:", self.level)
        print("Curve points:", self.points[1:self.level+2])
        print("Query point:", self.points[0])
        print("Result: dist=%s, X=%s, t=%s" % self.result)
        print("---")

    def recreate_points(self):
        self.canvas.delete("point")
        colors = ["red", "green", "blue", "blue", None]
        colors[self.level+1] = "#909"
        radius = 3
        for p, color in zip(self.points[:self.level+2], colors):
            coords = (p[0] - radius, p[1] - radius, p[0] + radius, p[1] + radius)
            self.canvas.create_oval(coords, fill=color, outline="white", tag="point")

    def refresh(self):
        self.canvas.delete("line")
        points = self.points[1:self.level+2]

        # Draw bbox
        if self.show_bbox:
            bbox_func = {1: line_bbox, 2: quadratic_bbox, 3: cubic_bbox}[self.level]
            bb_lt, bb_rb = bbox_func(*points)
            bb_rt = bb_rb[0], bb_lt[1]
            bb_lb = bb_lt[0], bb_rb[1]
            self.canvas.create_line(bb_lt, bb_rt, bb_rb, bb_lb, bb_lt,
                                    width=1, fill="blue", tag="line")

        if self.level == 1:
            self.canvas.create_line(points, width=2, fill="white", tag="line")
        else:
            self.canvas.create_line(points, width=1, fill="black", tag="line")
            curve_func = {2: quadratic_bezier, 3: cubic_bezier}[self.level]
            smoothness = 100
            start = points[0]
            for t in range(1, smoothness + 1):
                imp = curve_func(t / smoothness, *points)
                self.canvas.create_line(list(start), list(imp), width=2, fill="white", tag="line")
                start = imp

        px = self.points[0]
        dist_func = {1: line_distance, 2: quadratic_distance, 3: cubic_distance}[self.level]
        self.result = dist_func(px, *self.points[1:self.level+2])
        dist, x, _t = self.result
        self.label_dist.config(text="%s distance: %.2f" % (LEVEL_NAME[self.level], dist))
        self.canvas.create_line(list(x), px, width=2, fill="yellow", dash=(3,5), tag="line")

        # Bring points to front (above lines)
        self.canvas.tag_raise("point", "line")


if __name__ == '__main__':
    App().main()
