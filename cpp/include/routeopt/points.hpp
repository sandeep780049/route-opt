// Point parsing helpers shared by language ports that build native tools.
#ifndef ROUTEOPT_POINTS_HPP
#define ROUTEOPT_POINTS_HPP

#include <cctype>
#include <cmath>
#include <cstdio>
#include <string>
#include <vector>

namespace routeopt {

struct Point {
  int id;
  double x;
  double y;
};

// Render one scenario line: "id x y" with 1 decimal place, matching the
// corpus text format used by examples/*.txt.
inline std::string render_point(const Point &p) {
  char buf[64];
  std::snprintf(buf, sizeof buf, "%d %.1f %.1f", p.id, p.x, p.y);
  return std::string(buf);
}

// Parse "id x y" text into a Point; returns false on malformed input.
inline bool parse_point(const std::string &line, Point *out) {
  int id = 0;
  double x = 0.0, y = 0.0;
  if (std::sscanf(line.c_str(), "%d %lf %lf", &id, &x, &y) != 3) {
    return false;
  }
  out->id = id;
  out->x = x;
  out->y = y;
  return true;
}

// Sum of pairwise Euclidean distances over consecutive pairs (open path).
inline double path_length(const std::vector<Point> &pts) {
  double total = 0.0;
  for (std::size_t i = 1; i < pts.size(); ++i) {
    const double dx = pts[i].x - pts[i - 1].x;
    const double dy = pts[i].y - pts[i - 1].y;
    total += std::sqrt(dx * dx + dy * dy);
  }
  return total;
}

} // namespace routeopt

#endif // ROUTEOPT_POINTS_HPP
