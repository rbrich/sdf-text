#!/usr/bin/env python

import tkinter as tk
import math
import numpy as np
import numpy.linalg as la

WIDTH = 500
HEIGHT = 500

LEVEL_NAME = {
    2: "line",
    3: "quadratic bézier",
    4: "cubic bézier",
}


def ipairs(it):
    """Make new iterator which returns rolling window of pairs from `it`."""
    it = iter(it)
    a = next(it)
    while True:
        b = next(it)
        yield a, b
        a = b


def quadratic_bezier(t, p0, p1, p2):
    p0, p1, p2 = map(np.array, [p0, p1, p2])
    tc = 1 - t
    return tc*tc*p0 + 2*tc*t*p1 + t*t*p2


def quadratic_derivate(t, p0, p1, p2):
    p0, p1, p2 = map(np.array, [p0, p1, p2])
    tc = 1 - t
    return 2*tc*(p1 - p0) + 2*t*(p2 - p1)


def cubic_bezier(t, p0, p1, p2, p3):
    p0, p1, p2, p3 = map(np.array, [p0, p1, p2, p3])
    tc = 1 - t
    return tc*tc*tc*p0 + 3*tc*tc*t*p1 + 3*tc*t*t*p2 + t*t*t*p3


def line_distance(p, p0, p1):
    p0, p1, p = map(np.array, [p0, p1, p])
    a = p - p0
    b = p1 - p0
    t = a.dot(b) / b.dot(b)
    t = min(max(t, 0), 1)
    x = p0 + t * b
    # http://stackoverflow.com/a/1560510/6013024
    side = la.det([b, a])
    dist = math.copysign(la.norm(x - p), side)
    return dist, x


def solve_quadratic(a, b, c):
    """Find real roots of quadratic equation:

    a.x^2 + b.x + c = 0

    """
    D = b*b - 4*a*c
    if np.isclose(D, 0.0):
        return [-b/(2*a)]
    if D < 0:
        # Imaginary roots
        return []
    # D > 0
    Ds = math.sqrt(D)
    a2 = 2*a
    return [(-b + Ds)/a2, (-b - Ds)/a2]


def solve_cubic(a, b, c, d):
    """Find real roots of cubic equation:

    a.x^3 + b.x^2 + c.x + d = 0

    """
    # using numpy:
    return [r.real for r in np.poly1d([a, b, c, d]).r if r.imag == 0]
    # manual solution:
    if np.isclose(a, 0.0):
        return solve_quadratic(b, c, d)
    # normalized form (a => 1.0)
    b /= a
    c /= a
    d /= a
    # depressed form (t^3 + p.t + q = 0)
    p = c - b*b / 3
    q = 2/27 * b*b*b - b*b/3 + d
    if np.isclose(p, 0.0) and np.isclose(q, 0.0):
        return [0.0]
    q2 = q / 2
    p3 = p / 3
    D = q2 * q2 + p3 * p3 * p3
    if np.isclose(D, 0.0):
        x = math.pow(q2, 1/3)
        return [-2 * x, x]
    if D > 0:
        Ds = math.sqrt(D)
        return [math.pow(-q2 + Ds, 1/3) + math.pow(-q2 - Ds, 1/3)]
    # D < 0
    phi3 = math.acos(-q2 / math.sqrt(abs(p3)**3)) / 3
    x = 2 * math.sqrt(abs(p3))
    return [x*math.cos(phi3),
            -x*math.cos(phi3 - math.pi/3),
            -x*math.cos(phi3 + math.pi/3)]


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
    x_neareast = None
    for t, x in [(0.0, p0), (1.0, p2)]:
        dist = la.norm(x - p)
        if dist_min is None or dist < dist_min:
            dist_min = dist
            x_neareast = x
    for t in roots:
        if t < 0.0 or t > 1.0:
            continue
        x = quadratic_bezier(t, p0, p1, p2)
        dist = la.norm(x - p)
        if dist_min is None or dist < dist_min:
            dist_min = dist
            x_neareast = x
    return dist_min, x_neareast


def cubic_distance(p, p0, p1, p2, p3):
    # Basic numerical solution:
    # - evaluate set of points on the curve
    # - compute distance to a line formed from each pair of points
    p, p0, p1, p2, p3 = map(np.array, [p, p0, p1, p2, p3])
    steps = 30
    points = [p0] + [cubic_bezier(t / steps, p0, p1, p2, p3)
                     for t in range(1, steps)] + [p3]
    dist_min = None
    x_neareast = None
    for a, b in ipairs(points):
        dist, x = line_distance(p, a, b)
        if dist_min is None or abs(dist_min) > abs(dist):
            dist_min = dist
            x_neareast = x
    return dist_min, x_neareast


class App:

    def __init__(self):
        root = tk.Tk()
        self.label_help = tk.Label(root, text="[+/-] change curve level  [LMB] move point  [RMB] move view", justify=tk.LEFT)
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
        root.wm_title(string="Segment distance")

        self.level = 2
        self.points = [(WIDTH * 0.6, HEIGHT * 0.2),  # X point
                       (WIDTH * 0.2, HEIGHT * 0.4),  # P0
                       (WIDTH * 0.5, HEIGHT * 0.8),  # P1
                       (WIDTH * 0.8, HEIGHT * 0.4),  # P2
                       (WIDTH * 0.8, HEIGHT * 0.8)]  # P3
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

    def on_left_release(self, event):
        self.moving = None

    def on_right_press(self, event):
        self.scrolling = True
        self.canvas.scan_mark(event.x, event.y)

    def on_right_release(self, event):
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

    def on_key_plus(self, event):
        if self.level < 4:
            self.level += 1
            self.recreate_points()
            self.refresh()

    def on_key_minus(self, event):
        if self.level > 2:
            self.level -= 1
            self.recreate_points()
            self.refresh()

    def recreate_points(self):
        self.canvas.delete("point")
        color = "red"  # color for X point
        radius = 3
        for p in self.points[:self.level+1]:
            coords = (p[0] - radius, p[1] - radius, p[0] + radius, p[1] + radius)
            self.canvas.create_oval(coords, fill=color, outline="white", tag="point")
            color = "green"  # color for control points

    def refresh(self):
        self.canvas.delete("line")
        points = self.points[1:self.level+1]
        if self.level == 2:
            self.canvas.create_line(points, width=2, fill="white", tag="line")
        else:
            self.canvas.create_line(points, width=1, fill="black", tag="line")
            func = {3: quadratic_bezier, 4: cubic_bezier}[self.level]
            smoothness = 100
            start = points[0]
            for t in range(1, smoothness + 1):
                imp = func(t / smoothness, *points)
                self.canvas.create_line(list(start), list(imp), width=2, fill="white", tag="line")
                start = imp

        px = self.points[0]
        dist_func = {2: line_distance, 3: quadratic_distance, 4: cubic_distance}[self.level]
        dist, x = dist_func(px, *self.points[1:self.level+1])
        self.label_dist.config(text="%s distance: %.2f" % (LEVEL_NAME[self.level], dist))
        self.canvas.create_line(list(x), px, width=2, fill="yellow", dash=(3,5), tag="line")

        # Bring points to front (above lines)
        self.canvas.tag_raise("point", "line")


App().main()
