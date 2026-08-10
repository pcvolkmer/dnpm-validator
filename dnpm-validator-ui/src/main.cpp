#include <QApplication>
#include <QStyleFactory>
#include "mainwindow.h"

int main(int argc, char ** args) {
    QApplication app(argc, args);

    app.setStyle(QStyleFactory::create("windowsvista"));

    auto * mainwindow = new MainWindow();
    mainwindow->show();

    return app.exec();
}
