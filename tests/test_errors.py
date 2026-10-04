import pytest

import trendlib as tl


def test_hierarchy():
    assert issubclass(tl.InvalidInput, tl.TrendLibError)
    assert issubclass(tl.InsufficientHistory, tl.TrendLibError)
    assert issubclass(tl.InvalidInput, ValueError)
    assert issubclass(tl.InsufficientHistory, ValueError)
    assert not issubclass(tl.TrendLibError, ValueError)


@pytest.mark.parametrize("error", [tl.InvalidInput, tl.InsufficientHistory])
def test_caught_as_value_error(error):
    with pytest.raises(ValueError):
        raise error("boom")


def test_exported_names():
    assert set(tl.__all__) >= {
        "TrendLibError",
        "InvalidInput",
        "InsufficientHistory",
        "__version__",
        "__version_info__",
    }
