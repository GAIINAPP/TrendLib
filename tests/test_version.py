import trendlib as tl


def test_version_is_a_string():
    assert isinstance(tl.__version__, str)
    assert tl.__version__


def test_version_info_is_a_tuple_of_ints():
    assert isinstance(tl.__version_info__, tuple)
    assert tl.__version_info__
    assert all(isinstance(part, int) for part in tl.__version_info__)


def test_version_info_matches_version():
    assert tl.__version__.startswith(".".join(str(p) for p in tl.__version_info__))


def test_core_module_is_not_part_of_the_public_surface():
    assert "_core" not in tl.__all__


def test_version_info_drops_a_prerelease_suffix():
    from trendlib import _version_info

    assert _version_info("0.0.0") == (0, 0, 0)
    assert _version_info("0.1.0a1") == (0, 1, 0)
    assert _version_info("1.2.3") == (1, 2, 3)
