package dev.routeopt;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

/**
 * Plane-sweep closest-pair search over scenario point clouds.
 *
 * <p>Used by the verification pipeline as an independent estimator of the
 * smallest inter-point distance, so degenerate corpora (duplicated points,
 * near-collinear sets) are detected before any solver runs on them.
 */
public final class PlaneSweep {

    /** Minimum pairwise distance; {@code Double.POSITIVE_INFINITY} when n &lt; 2. */
    public static double closestPair(Pt[] points) {
        if (points == null || points.length < 2) {
            return Double.POSITIVE_INFINITY;
        }
        Pt[] sorted = points.clone();
        Arrays.sort(sorted, (a, b) -> Double.compare(a.x, b.x) != 0
                ? Double.compare(a.x, b.x)
                : Double.compare(a.y, b.y));
        return sweep(sorted, 0, sorted.length - 1);
    }

    private static double sweep(Pt[] sorted, int lo, int hi) {
        if (hi - lo < 3) {
            double best = Double.POSITIVE_INFINITY;
            for (int i = lo; i <= hi; i++) {
                for (int j = i + 1; j <= hi; j++) {
                    best = Math.min(best, sorted[i].distanceTo(sorted[j]));
                }
            }
            return best;
        }
        int mid = (lo + hi) >>> 1;
        double dLeft = sweep(sorted, lo, mid);
        double dRight = sweep(sorted, mid + 1, hi);
        double d = Math.min(dLeft, dRight);

        List<Pt> strip = new ArrayList<>();
        double midX = sorted[mid].x;
        for (int i = lo; i <= hi; i++) {
            if (Math.abs(sorted[i].x - midX) < d) {
                strip.add(sorted[i]);
            }
        }
        strip.sort((a, b) -> Double.compare(a.y, b.y));
        for (int i = 0; i < strip.size(); i++) {
            for (int j = i + 1; j < strip.size(); j++) {
                if (strip.get(j).y - strip.get(i).y >= d) {
                    break;
                }
                d = Math.min(d, strip.get(i).distanceTo(strip.get(j)));
            }
        }
        return d;
    }

    /** Immutable planar point. */
    public record Pt(int id, double x, double y) {
        public double distanceTo(Pt other) {
            double dx = x - other.x;
            double dy = y - other.y;
            return Math.sqrt(dx * dx + dy * dy);
        }
    }

    private PlaneSweep() {
        // utility container
    }
}
