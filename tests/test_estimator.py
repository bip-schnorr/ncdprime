from ncdprime.estimator import CompletionTimeEstimator, TimingSample


def test_linear_fit_reasonable():
    est = CompletionTimeEstimator()
    # t = 0.05 + 2e-6 * n
    for n in [10_000, 50_000, 100_000, 200_000, 400_000, 800_000]:
        est.add_sample(TimingSample(input_bytes=n, wall_time_s=0.05 + 2e-6 * n))

    fit = est.fit_from_first_n(6)
    assert fit is not None
    assert fit.model in ("linear", "power")

    # Prediction should be close for an in-range point
    pred = est.predict_time_s(300_000)
    assert pred is not None
    assert abs(pred - (0.05 + 2e-6 * 300_000)) < 0.05


def test_power_fit_reasonable():
    est = CompletionTimeEstimator()
    # t = 1e-4 * n^0.9
    for n in [5_000, 20_000, 80_000, 200_000, 600_000, 1_200_000]:
        est.add_sample(TimingSample(input_bytes=n, wall_time_s=1e-4 * (n ** 0.9)))

    fit = est.fit_from_first_n(6)
    assert fit is not None

    pred = est.predict_time_s(500_000)
    assert pred is not None
    # loose tolerance
    assert 0.5 <= pred / (1e-4 * (500_000 ** 0.9)) <= 1.5


def test_estimate_remaining_sums_predictions():
    est = CompletionTimeEstimator()
    for n in [10_000, 20_000, 30_000, 40_000, 50_000, 60_000]:
        est.add_sample(TimingSample(input_bytes=n, wall_time_s=0.001 * n))
    est.fit_from_first_n(6)

    remaining = [1_000, 2_000, 3_000]
    total = est.estimate_remaining_s(remaining)
    assert total is not None
    assert abs(total - 0.001 * sum(remaining)) < 1e-6
