import pathlib
import os.path


PATH_PACKAGE_DATA = os.path.join(pathlib.Path(__file__).parent.parent, "rulesets")

PATH_RULESETS = (
    os.path.join(PATH_PACKAGE_DATA, "tracking_params.json"),
)

IGNORED_PROVIDERS = (
    "ClearURLsTest",
    "ClearURLsTestBlock",
    "ClearURLsTest2",
    "ClearURLsTestBlock2"
)
