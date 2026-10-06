package dev.routeopt;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Locale;

import java.util.random.RandomGenerator;
import java.util.random.RandomGeneratorFactory;
import org.junit.jupiter.api.Test;

class PlaneSweepTest {

    private static PlaneSweep.Pt pt(int id, double x, double y) {
        return new PlaneSweep.Pt(id, x, y);
    }

    @Test
    void infinityForFewerThanTwoPoints() {
        assertEquals(Double.POSITIVE_INFINITY, PlaneSweep.closestPair(new PlaneSweep.Pt[] {}));
        assertEquals(Double.POSITIVE_INFINITY,
                PlaneSweep.closestPair(new PlaneSweep.Pt[] { pt(1, 0, 0) }));
    }

    @Test
    void knownTriangle() {
        PlaneSweep.Pt[] triangle = { pt(0, 0, 0), pt(1, 3, 4), pt(2, 0, 1) };
        assertEquals(1.0, PlaneSweep.closestPair(triangle));
    }

    @Test
    void matchesBruteForceOnRandomClouds() {
        var rng = RandomGeneratorFactory.of("L64X128MixRandom").create(2026);
        for (int trial = 0; trial < 25; trial++) {
            int n = 20 + rng.nextInt(80);
            PlaneSweep.Pt[] cloud = new PlaneSweep.Pt[n];
            for (int i = 0; i < n; i++) {
                cloud[i] = pt(i, rng.nextDouble(-100, 100), rng.nextDouble(-100, 100));
            }
            double brute = Double.POSITIVE_INFINITY;
            for (int i = 0; i < n; i++) {
                for (int j = i + 1; j < n; j++) {
                    double d = cloud[i].distanceTo(cloud[j]);
                    if (d < brute) {
                        brute = d;
                    }
                }
            }
            double swept = PlaneSweep.closestPair(cloud);
            assertEquals(brute, swept, 1e-9,
                    String.format(Locale.ROOT, "trial %d: brute=%s swept=%s",
                            trial, brute, swept));
        }
    }

    @Test
    void tightPairInLargeCloud() {
        var rng = RandomGeneratorFactory.of("L64X128MixRandom").create(7);
        PlaneSweep.Pt[] cloud = new PlaneSweep.Pt[500];
        for (int i = 0; i < cloud.length; i++) {
            cloud[i] = pt(i, rng.nextDouble(0, 1000), rng.nextDouble(0, 1000));
        }
        cloud[317] = new PlaneSweep.Pt(317, 500.0, 500.0);
        cloud[481] = new PlaneSweep.Pt(481, 500.01, 500.0);
        assertTrue(PlaneSweep.closestPair(cloud) <= 0.01);
    }
}
