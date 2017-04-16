def pytest_addoption(parser):
    parser.addoption("--url", action="store", default='http://localhost', help="Host to run tests against")
    parser.addoption("--port", action="store", default='8000', help="Port where webapp is running")

def pytest_generate_tests(metafunc):
    if 'url' in metafunc.fixturenames:
        metafunc.parametrize("url", [metafunc.config.option.url])
    if 'port' in metafunc.fixturenames:
        metafunc.parametrize("port", [metafunc.config.option.port])
