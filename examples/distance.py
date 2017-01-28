#!/usr/bin/env python

import tkinter as tk
import math
import numpy as np
import numpy.linalg as la

WIDTH = 500
HEIGHT = 500


class App:

    def __init__(self):
        self.canvas = tk.Canvas(width=WIDTH, height=HEIGHT, bg="gray")
        self.canvas.pack(fill=tk.BOTH, expand=True)
        self.label = tk.Label()
        self.label.pack()
        self.canvas.bind("<ButtonPress-1>", self.on_left_press)
        self.canvas.bind("<ButtonRelease-1>", self.on_left_release)
        self.canvas.bind("<ButtonPress-3>", self.on_right_press)
        self.canvas.bind("<ButtonRelease-3>", self.on_right_release)
        self.canvas.bind("<Motion>", self.on_move)
        self.canvas.master.wm_title(string="Segment distance")

        points = [(WIDTH * 0.3, HEIGHT * 0.3), (WIDTH * 0.7, HEIGHT * 0.7)]
        point_x = (WIDTH * 0.6, HEIGHT * 0.2)
        self.points = [self.create_point(p) for p in points]
        self.point_x = self.create_point(point_x, "green")
        self.segment = None
        self.line_x = None
        self.refresh()

        self.moving = None
        self.movable = self.points + [self.point_x]
        self.scrolling = False

    def main(self):
        tk.mainloop()

    def on_left_press(self, event):
        mx, my = self.canvas.canvasx(event.x), self.canvas.canvasy(event.y)
        tolerance = 6
        items = self.canvas.find_closest(mx, my)
        for item in items:
            if item in self.movable:
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
            x, y = self.point_coords(item)
            rx = self.canvas.canvasx(event.x) - x
            ry = self.canvas.canvasy(event.y) - y
            self.canvas.move(item, rx, ry)
            self.refresh()
        if self.scrolling:
            self.canvas.scan_dragto(event.x, event.y, 1)

    def point_coords(self, item):
        x1, y1, x2, y2 = self.canvas.coords(item)
        x = x1 + (x2 - x1) / 2
        y = y1 + (y2 - y1) / 2
        return x, y

    def refresh(self):
        if self.segment is not None:
            self.canvas.delete(self.segment)
        points = [self.point_coords(item) for item in self.points]
        self.segment = self.canvas.create_line(points, width=2, fill="white")

        px = self.point_coords(self.point_x)
        dist, x = self.signed_distance(px)
        self.label.config(text="distance: %03f" % dist)
        if self.line_x is not None:
            self.canvas.delete(self.line_x)
        self.line_x = self.canvas.create_line(list(x), px, width=2, fill="yellow", dash=(3,5))
        self.canvas.tag_raise("point", self.line_x)

    def create_point(self, p, color="red"):
        radius = 3
        coords = (p[0] - radius, p[1] - radius, p[0] + radius, p[1] + radius)
        return self.canvas.create_oval(coords, fill=color, outline="white", tag="point")

    def signed_distance(self, p):
        p0, p1 = [self.point_coords(item) for item in self.points]
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


App().main()
